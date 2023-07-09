use std::{fs::File, io::{BufReader, Read, BufWriter, Write}};

mod hvqm;
mod adpcm;


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
    print!("H sampling rate     : {}\n", hvqm_header.h_sampling_rate);
    print!("V sampling rate     : {}\n", hvqm_header.v_sampling_rate);
    print!("Compress type       : {}\n", if hvqm_header.v_sampling_rate == 1 { "4:2:2" } else { "4:1:1" });
    print!("Y shiftnum          : {}\n", hvqm_header.y_shiftnum);
    print!("Video quantized step: {}\n", hvqm_header.video_quantize_shift);
    print!("Total frames        : {}\n", hvqm_header.total_frames);
    print!("Frame interval      : {} usec\n", hvqm_header.usec_per_frame);
    print!("Video rate          : {} frame/sec\n", 1000000.0 / hvqm_header.usec_per_frame as f32);
    print!("Max frame size      : {} bytes\n", hvqm_header.max_frame_size);
    print!("Max SP packets      : {} bytes\n", hvqm_header.max_sp_packets);
    print!("Audio data format   : {}\n", hvqm_header.audio_format);
    print!("Audio channels      : {}\n", hvqm_header.channels);
    print!("Bits per sample     : {} bit\n", hvqm_header.sample_bits);
    print!("Audio quantized step: {}\n", hvqm_header.audio_quantize_step);
    print!("Total audio records : {}\n", hvqm_header.total_audio_records);
    print!("Audio rate          : {} Hz\n", hvqm_header.samples_per_sec);
    print!("Max audio record    : {} bytes\n", hvqm_header.max_audio_record_size);
    print!("\n");
    print!("Display mode        : {}\n", "16-bit RGBA");
    print!("\n");

    let mut adpcm_state = adpcm::ADPCMstate::new();

    let mut record_index = 0;

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
        println!("size        = 0x{:X} bytes", record.size);
        println!();

        match record_type {
            hvqm::RecordType::Audio => {
                let audio_header = hvqm::HVQM2AudioHeader::new(&input_buf[offset..]);
                let mut pcmbuf: [i16; 11988] = [0; 11988];

                println!("    samples     = {}", audio_header.samples);

                adpcm_state.adpcmDecode(&input_buf[offset+4..], record_format.toADPCMFormat().expect("idk"), audio_header.samples, &mut pcmbuf, true);

                let output_file = File::create(format!("audio_record_{record_index:04}.pcm_raw")).expect("could not create output file");

                let mut pcmbuf_byte = Vec::new();
                for value in pcmbuf {
                    pcmbuf_byte.extend(value.to_be_bytes());
                }
                BufWriter::new(output_file).write(&pcmbuf_byte).expect("Could not write to output file");
            },
            // hvqm::RecordType::Video => (),
            _ => (),
        }

        println!();

        offset += record.size as usize;
        record_index += 1;
    }
}
