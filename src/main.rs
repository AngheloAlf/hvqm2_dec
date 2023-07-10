use std::{fs::File, io::{BufReader, Read, BufWriter, Write}};
use clap::Parser;

mod hvqm;
mod adpcm;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input HVQM file
    input: String,

    /// Print record information while processing HVQM file
    #[arg(long)]
    print_record_info: bool,
}

fn main() {
    let args = Args::parse();

    let input_path = &args.input;
    let print_record_info = args.print_record_info;

    let input_file = File::open(input_path).expect("could not open input file");
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
    let mut audio_record_count = 0;
    let mut video_record_count = 0;

    let mut compressed_audio_size = 0;
    let mut decoded_audio_bytes = Vec::new();
    let mut decoded_audio_halfs = Vec::new();

    let mut offset = 0x3C;
    loop {
        if offset >= input_buf.len() {
            break;
        }

        let record = hvqm::HVQM2Record::new(&input_buf[offset..]);

        offset += 0x8;

        let record_type = record.record_type().expect("oy noy");
        let record_format = record.data_format().expect("nyoron");

        if print_record_info {
            println!("record_type = {:#?}", record_type);
            println!("format      = {:#?}", record_format);
            println!("size        = 0x{:X} bytes", record.size);
            println!();
        }

        match record_type {
            hvqm::RecordType::Audio => {
                let audio_header = hvqm::HVQM2AudioHeader::new(&input_buf[offset..]);
                let mut pcmbuf: [i16; 11988] = [0; 11988];

                if print_record_info {
                    println!("    samples     = {}", audio_header.samples);
                }

                adpcm_state.adpcmDecode(&input_buf[offset+4..], record_format.toADPCMFormat().expect("idk"), audio_header.samples, &mut pcmbuf, false);

                let mut pcmbuf_byte = Vec::new();
                let mut i = 0;
                while i < audio_header.samples {
                    let value = pcmbuf[i as usize];
                    let value_bytes = value.to_be_bytes();

                    pcmbuf_byte.extend(value_bytes);
                    decoded_audio_bytes.extend(value_bytes);
                    decoded_audio_halfs.push(value);
                    i += 1;
                }

                // let output_file = File::create(format!("audio_record_{record_index:04}.pcm_raw")).expect("could not create output file");
                // BufWriter::new(output_file).write(&pcmbuf_byte).expect("Could not write to output file");

                compressed_audio_size += record.size;
                audio_record_count += 1;
            },
            hvqm::RecordType::Video => {
                let mut suboffset = 0;

                // TODO: handle HOLD better

                if print_record_info {
                    match record_format {
                        hvqm::DataFormat::VideoKeyframe | hvqm::DataFormat::VideoPredict => {
                            let video_header = hvqm::HVQM2Frame::new(&input_buf[offset..]);
                            suboffset += 0x34;

                            println!("    basisnum_offset[0] = {}", video_header.basisnum_offset[0]);
                            println!("    basisnum_offset[1] = {}", video_header.basisnum_offset[1]);
                            println!("    basnumrn_offset[0] = {}", video_header.basnumrn_offset[0]);
                            println!("    basnumrn_offset[1] = {}", video_header.basnumrn_offset[1]);
                            println!("    scale_offset[0]    = {}", video_header.scale_offset[0]);
                            println!("    scale_offset[1]    = {}", video_header.scale_offset[1]);
                            println!("    scale_offset[2]    = {}", video_header.scale_offset[2]);
                            println!("    fixvl_offset[0]    = {}", video_header.fixvl_offset[0]);
                            println!("    fixvl_offset[1]    = {}", video_header.fixvl_offset[1]);
                            println!("    fixvl_offset[2]    = {}", video_header.fixvl_offset[2]);
                            println!("    dcval_offset[0]    = {}", video_header.dcval_offset[0]);
                            println!("    dcval_offset[1]    = {}", video_header.dcval_offset[1]);
                            println!("    dcval_offset[2]    = {}", video_header.dcval_offset[2]);
                        },
                        hvqm::DataFormat::VideoHold => (),
                        _  => panic!("what"),
                    }

                }

                match record_format {
                    hvqm::DataFormat::VideoKeyframe => {
                        let key_header = hvqm::HVQM2KeyFrame::new(&input_buf[offset+suboffset..]);
                        suboffset += 0x14;

                        if print_record_info {
                            println!("        dcrun_offset[0] = {}", key_header.dcrun_offset[0]);
                            println!("        dcrun_offset[1] = {}", key_header.dcrun_offset[1]);
                            println!("        dcrun_offset[2] = {}", key_header.dcrun_offset[2]);
                            println!("        nest_start_x    = {}", key_header.nest_start_x);
                            println!("        nest_start_y    = {}", key_header.nest_start_y);
                        }
                    },
                    hvqm::DataFormat::VideoPredict => {
                        let predict_header = hvqm::HVQM2PredictFrame::new(&input_buf[offset+suboffset..]);
                        suboffset += 0x8;

                        if print_record_info {
                            println!("        movevector_offset    = {}", predict_header.movevector_offset);
                            println!("        macroblock_offset    = {}", predict_header.macroblock_offset);
                        }
                    },
                    hvqm::DataFormat::VideoHold => (),
                    _  => panic!("what"),
                }

                // println!();
                // println!("    size remaining: {}", record.size as i32 - suboffset as i32);
                // println!();

                video_record_count += 1;
            },
            _ => (),
        }

        if print_record_info {
            println!();
        }

        offset += record.size as usize;
        record_index += 1;
    }

    // let output_file = File::create(format!("{input_path}.pcm_raw")).expect("could not create output file");
    // BufWriter::new(output_file).write(&decoded_audio_bytes).expect("Could not write to output file");

    let mut out_wav_file = File::create(format!("{input_path}.wav")).expect("not");

    let wav_header = wav::Header::new(wav::header::WAV_FORMAT_PCM, hvqm_header.channels as u16, hvqm_header.samples_per_sec, hvqm_header.sample_bits as u16);
    let wav_bitdepth = wav::bit_depth::BitDepth::Sixteen(decoded_audio_halfs);
    wav::write(wav_header, &wav_bitdepth, &mut out_wav_file).expect("error when writing wav file");

    println!("compressed_audio_size = {compressed_audio_size}");
    println!("audio_record_count    = {audio_record_count}");
    println!("video_record_count    = {video_record_count}");
}
