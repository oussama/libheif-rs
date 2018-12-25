#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn it_works() -> Result<(), Error> {
        let mut ctx = Context::new()?;
        println!("ctx created");
        ctx.read_from_file("./data/test.HEIC")?;
        println!("file created");
        // get a handle to the primary image
        let handle = ctx.get_primary_image_handle()?;

        println!("handle created {:?}x{:?}", handle.width(), handle.height());

        // decode the image and convert colorspace to RGB, saved as 24bit interleaved
        let img = handle.decode(&DecodeOptions::new())?;
        println!(
            "img created {:?}x{:?} {:?} {:?}",
            img.width(),
            img.height(),
            img.get_chroma_format(),
            img.get_color_space()
        );
        /*
                    match img.get_color_space() {
                        ColorSpace::YCbCr => {
                            let subsample = match img.get_chroma_format() {
                                Chroma::C420
                            }
                        },
                        _ => unimplemented!(),
                    }
        */
        let (bytes_y, stride_y) = img.get_plane(Channel::Y);

        let (bytes_u, stride_u) = img.get_plane(Channel::Cb);

        let (bytes_v, stride_v) = img.get_plane(Channel::Cr);

        let mut compress = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_YCbCr);
        compress.set_scan_optimization_mode(mozjpeg::ScanMode::AllComponentsTogether);
        compress.set_size(img.width() as _, img.height() as _);
        compress.set_mem_dest();
        compress.start_compress();

        for y in 0..img.height() {
            let mut bytes = Vec::with_capacity((img.width() as usize) * 3);
            for x in 0..(img.width() as usize) {
                let offset_y = (y * stride_y) as usize;
                bytes.push(bytes_y[offset_y + x]);
                let offset_u = ((y / 2) * stride_u) as usize;
                bytes.push(bytes_u[offset_u + x / 2]);
                let offset_v = ((y / 2) * stride_v) as usize;
                bytes.push(bytes_v[offset_v + x / 2]);
            }
            compress.write_scanlines(&bytes);
        }
        compress.finish_compress();

        use std::fs::write;
        let _ = write("./data/out.jpg", &compress.data_to_vec().unwrap());
        Ok(())
    }
}
