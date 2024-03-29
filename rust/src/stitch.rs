
use crate::{Error, Logger, Options};
use jni::{objects::AutoElements, sys::jint};
use stitchy_core::{
    Stitch, ImageFiles, OwnedRawFdLocation, FileLocation, FileProperties,
    image::{ImageFormat, ImageOutputFormat}
};

pub fn run_stitchy(
    mut logger: Logger,
    options: Options,
    input_fds: AutoElements<jint>,
    input_mimes: Vec<String>,
    output_fd: jint,
    output_mime: String
) -> Result<(), Error> {

    let mut images_builder = ImageFiles::builder();

    for (i, fd) in input_fds.into_iter().enumerate() {
        if i >= input_mimes.len() {
            return Err(Error::Unknown("Internal error: mismatch in file data".to_owned()));
        }
        let location = OwnedRawFdLocation::new(*fd, input_mimes[i].to_owned());
        images_builder = images_builder.add_file(location)?;
        logger.log_message("File added to ImageFiles")?;
    }

    let images = images_builder.build()
        .map_err(|e| Error::Unknown(e))?;

    let stitch_output = Stitch::builder()
        .alignment(options.get_alignment())
        .width_limit(options.maxw as u32)
        .height_limit(options.maxh as u32)
        .image_files(images)?
        .stitch()?;

    logger.log_message("Stitch completed")?;

    let mut output_properties = OwnedRawFdLocation::new(output_fd, output_mime)
        .into_properties().map_err(|e| Error::Unknown(e))?;
    let requested_output_format = output_properties
        .infer_format()
        .ok_or_else(|| Error::Unknown("Could not determine output format".to_owned()))?;
    let output_format = make_image_output_format(requested_output_format, options.quality)?;

    let mut output_file = output_properties.borrow_file_mut();
    stitch_output.write_to(&mut output_file, output_format)?;

    logger.log_message("Output written; Stitchy completed successfully")?;

    Ok(())
}

fn make_image_output_format(format: ImageFormat, quality: u32) -> Result<ImageOutputFormat, Error> {
    match format {
        ImageFormat::Jpeg => Ok(ImageOutputFormat::Jpeg(quality as u8)),
        ImageFormat::Png => Ok(ImageOutputFormat::Png),
        ImageFormat::Gif => Ok(ImageOutputFormat::Gif),
        ImageFormat::Bmp => Ok(ImageOutputFormat::Bmp),
        _ => Err(Error::Unknown(format!("Unexpected output format: {:?}", format)))
    }
}
