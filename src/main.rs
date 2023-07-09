use std::{fs::File, io::{BufReader, Read}};

mod hvqm;


fn main() {
    let input_file = File::open("YOULOSE.HVQM").expect("could not open input file");
    let mut input_buf = Vec::new();
    BufReader::new(input_file).read_to_end(&mut input_buf).expect("error");

    let hvqm_header = hvqm::HVQM2Header::new(&input_buf);

    if !hvqm_header.valid_header() {
        panic!("invalid header");
    }

    print!("\n");
    print!("File version        : {}\n", hvqm_header.header_str());
    print!("File size           : {}\n", hvqm_header.file_size);
    print!("Image width         : {}\n", hvqm_header.width);
    print!("Image height        : {}\n", hvqm_header.height);
    print!("Compress type       : {}\n", if hvqm_header.v_sampling_rate == 1 { "4:2:2" } else { "4:1:1" });
    print!("Total frames        : {}\n", hvqm_header.total_frames);
    print!("Video rate          : {} frame/sec\n", 1000000.0 / hvqm_header.usec_per_frame as f32);
    print!("Total audio records : {}\n", hvqm_header.total_audio_records);
    print!("Audio rate          : {} Hz\n", hvqm_header.samples_per_sec);
    print!("\n");
    print!("Display mode        : {}\n", "16-bit RGBA");
    print!("\n");

    let mut offset = 0x3C;
    loop {
        if offset >= input_buf.len() {
            break;
        }

        let record = hvqm::HVQM2Record::new(&input_buf[offset..]);

        offset += 0x8;

        let record_type = record.record_type().expect("oy noy");
        let record_format = record.data_format().expect("nyoron");

        println!("record_type = {:#?}", record_type);
        println!("format      = {:#?}", record_format);
        println!("size        = 0x{:X}", record.size);
        println!();

        offset += record.size as usize;
    }
}
