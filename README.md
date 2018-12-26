# libheif

Safe wrapper to libheif-dev for parsing heif (heic) files.

## Example
```
        let mut ctx = Context::new()?;
        ctx.read_from_file("./data/test.HEIC")?;
        // get a handle to the primary image
        let handle = ctx.get_primary_image_handle()?;
        // decode the image
        let img = handle.decode(&DecodeOptions::new())?;

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
        write(
            "./target/out.jpg",
            &compress.data_to_vec().expect("data to vec"),
        )?;
```