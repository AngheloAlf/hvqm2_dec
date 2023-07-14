/*
 * HVQM2Header : HVQM2 file header
 */
pub struct HVQM2Header {
    /* 0x00 */ pub file_version: [u8; 16],
    /* 0x10 */ pub file_size: u32,              /* File size [byte] */

    /* 0x14 */ pub width: u16,                  /* Number of pixels in horizontal direction of image */
    /* 0x16 */ pub height: u16,                 /* Number of pixels in vertical direction of image */
    /* 0x18 */ pub h_sampling_rate: u8,         /* Video UV component's sampling step in horizontal direction */
    /* 0x19 */ pub v_sampling_rate: u8,         /* Video UV component's sampling step in vertical direction */
    /* 0x1A */ pub y_shiftnum: u8,              /* Image base read start y-coordinate LSB */
    /* 0x1B */ pub video_quantize_shift: u8,    /* Video quantized step */

    /* 0x1C */ pub total_frames: u32,      /* Total number of video records  */
    /* 0x20 */ pub usec_per_frame: u32,    /* Video frame interval [usec.] */
    /* 0x24 */ pub max_frame_size: u32,    /* Maximum size of video record [bytes] (Excluding record header) */
    /* 0x28 */ pub max_sp_packets: u32,    /* Maximum number of packets needed for SP FIFO */

    /* 0x2C */ pub audio_format: u8,           /* Audio data format  */
    /* 0x2D */ pub channels: u8,               /* Number of audio channels  */
    /* 0x2E */ pub sample_bits: u8,            /* Number of bits in 1 sample (channel) [bit] */
    /* 0x2F */ pub audio_quantize_step: u8,    /* Audio quantized step */

    /* 0x30 */ pub total_audio_records: u32,      /* Total number of audio records  */
    /* 0x34 */ pub samples_per_sec: u32,          /* Number of audio samples per second */
    /* 0x38 */ pub max_audio_record_size: u32,    /* Maximum size of audio record [byte] (Excluding record header) */
}

impl HVQM2Header {
    pub fn new(buf: &Vec<u8>) -> HVQM2Header {
        let file_version: [u8; 0x10] = buf[0x00..0x10].try_into().unwrap();
        let file_size = u32::from_be_bytes(buf[0x10..0x14].try_into().unwrap());

        let width = u16::from_be_bytes(buf[0x14..0x16].try_into().unwrap());
        let height = u16::from_be_bytes(buf[0x16..0x18].try_into().unwrap());
        let h_sampling_rate = u8::from_be_bytes(buf[0x18..0x19].try_into().unwrap());
        let v_sampling_rate = u8::from_be_bytes(buf[0x19..0x1A].try_into().unwrap());
        let y_shiftnum = u8::from_be_bytes(buf[0x1A..0x1B].try_into().unwrap());
        let video_quantize_shift = u8::from_be_bytes(buf[0x1B..0x1C].try_into().unwrap());

        let total_frames = u32::from_be_bytes(buf[0x1C..0x20].try_into().unwrap());
        let usec_per_frame = u32::from_be_bytes(buf[0x20..0x24].try_into().unwrap());
        let max_frame_size = u32::from_be_bytes(buf[0x24..0x28].try_into().unwrap());
        let max_sp_packets = u32::from_be_bytes(buf[0x28..0x2C].try_into().unwrap());

        let audio_format = u8::from_be_bytes(buf[0x2C..0x2D].try_into().unwrap());
        let channels = u8::from_be_bytes(buf[0x2D..0x2E].try_into().unwrap());
        let sample_bits = u8::from_be_bytes(buf[0x2E..0x2F].try_into().unwrap());
        let audio_quantize_step = u8::from_be_bytes(buf[0x2F..0x30].try_into().unwrap());

        let total_audio_records = u32::from_be_bytes(buf[0x30..0x34].try_into().unwrap());
        let samples_per_sec = u32::from_be_bytes(buf[0x34..0x38].try_into().unwrap());
        let max_audio_record_size = u32::from_be_bytes(buf[0x38..0x3C].try_into().unwrap());

        HVQM2Header {
            file_version: file_version,
            file_size: file_size,
            width: width,
            height: height,
            h_sampling_rate: h_sampling_rate,
            v_sampling_rate: v_sampling_rate,
            y_shiftnum: y_shiftnum,
            video_quantize_shift: video_quantize_shift,
            total_frames: total_frames,
            usec_per_frame: usec_per_frame,
            max_frame_size: max_frame_size,
            max_sp_packets: max_sp_packets,
            audio_format: audio_format,
            channels: channels,
            sample_bits: sample_bits,
            audio_quantize_step: audio_quantize_step,
            total_audio_records: total_audio_records,
            samples_per_sec: samples_per_sec,
            max_audio_record_size: max_audio_record_size,
        }
    }

    pub fn valid_header(&self) -> bool {
        let valid: [u8; 0x10] = [0x48, 0x56, 0x51, 0x4D, 0x32, 0x20, 0x31, 0x2E, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,];

        self.file_version == valid
    }

    pub fn header_str(&self) -> &str {
        std::str::from_utf8(&self.file_version).unwrap()
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RecordType {
    Audio,
    Video,
}

impl RecordType {
    pub fn from_u16(t: u16) -> Result<RecordType, ()> {
        match t {
            0 => Ok(RecordType::Audio),
            1 => Ok(RecordType::Video),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DataFormat {
    AudioKeyframe,
    AudioPredict,
    VideoKeyframe,
    VideoPredict,
    VideoHold,
}

impl DataFormat {
    pub fn from_u16(format: u16, record_type: RecordType) -> Result<DataFormat, ()> {
        match record_type {
            RecordType::Audio => match format {
                0 => Ok(DataFormat::AudioKeyframe),
                1 => Ok(DataFormat::AudioPredict),
                _ => Err(())
            },
            RecordType::Video => match format {
                0 => Ok(DataFormat::VideoKeyframe),
                1 => Ok(DataFormat::VideoPredict),
                2 => Ok(DataFormat::VideoHold),
                _ => Err(())
            },
        }
    }

    pub fn toADPCMFormat(&self) -> Result<crate::adpcm::ADPCMFormat, ()> {
        match self {
            DataFormat::AudioKeyframe => Ok(crate::adpcm::ADPCMFormat::Reset),
            DataFormat::AudioPredict => Ok(crate::adpcm::ADPCMFormat::Continue),
            _ => Err(()),
        }
    }
}


pub struct HVQM2Record {
    pub r_type: u16,          /* Record type  */
    pub format: u16,          /* Data format  */
    pub size: u32,            /* Record size (excluding the header) [byte] */
}

/*
 * HVQM2Record : Record header (Located directly after HVQM2Header)
 */
impl HVQM2Record {
    pub fn new(buf: &[u8]) -> HVQM2Record {
        let r_type = u16::from_be_bytes(buf[0x0..0x2].try_into().unwrap());
        let format = u16::from_be_bytes(buf[0x2..0x4].try_into().unwrap());
        let size = u32::from_be_bytes(buf[0x4..0x8].try_into().unwrap());

        HVQM2Record {
            r_type : r_type,
            format : format,
            size : size,
        }
    }

    pub fn record_type(&self) -> Result<RecordType, ()> {
        RecordType::from_u16(self.r_type)
    }

    pub fn data_format(&self) -> Result<DataFormat, ()> {
        match self.record_type() {
            Err(value) => Err(value),
            Ok(rec_type) => DataFormat::from_u16(self.format, rec_type)
        }
    }
}

/*
 * HVQM2Audio : Audio header (Follows record header)
 */
pub struct HVQM2AudioHeader {
    pub samples: u32,        /* Number of samples (/channels)  */
}

impl HVQM2AudioHeader {
    pub fn new(buf: &[u8]) -> HVQM2AudioHeader {
        let samples = u32::from_be_bytes(buf[0x0..0x4].try_into().unwrap());

        HVQM2AudioHeader {
            samples : samples,
        }
    }
}

/*
 * HVQM2Frame :  Video header  (Follows record header)
 */
pub struct HVQM2Frame {
    /* 0x00 */ pub basisnum_offset: [u32; 2],    /* Basis number block (0: brightness, 1: color difference) */
    /* 0x08 */ pub basnumrn_offset: [u32; 2],    /* Basis number cold run (0: brightness, 1: color difference)   */
    /* 0x10 */ pub scale_offset: [u32; 3],    /* Basis coefficient (0:Y, 1:U, 2:V) */
    /* 0x1C */ pub fixvl_offset: [u32; 3],    /* Fixed length code (0:Y, 1:U, 2:V) */
    /* 0x28 */ pub dcval_offset: [u32; 3],    /* Block DC (0:Y, 1:U, 2:V) */
}

impl HVQM2Frame {
    pub fn new(buf: &[u8]) -> HVQM2Frame {
        let basisnum_offset: [u32; 2] = [u32::from_be_bytes(buf[0x00..0x04].try_into().unwrap()), u32::from_be_bytes(buf[0x04..0x08].try_into().unwrap())];
        let basnumrn_offset: [u32; 2] = [u32::from_be_bytes(buf[0x08..0x0C].try_into().unwrap()), u32::from_be_bytes(buf[0x0C..0x10].try_into().unwrap())];
        let scale_offset: [u32; 3] = [u32::from_be_bytes(buf[0x10..0x14].try_into().unwrap()), u32::from_be_bytes(buf[0x14..0x18].try_into().unwrap()), u32::from_be_bytes(buf[0x18..0x1C].try_into().unwrap())];
        let fixvl_offset: [u32; 3] = [u32::from_be_bytes(buf[0x1C..0x20].try_into().unwrap()), u32::from_be_bytes(buf[0x20..0x24].try_into().unwrap()), u32::from_be_bytes(buf[0x24..0x28].try_into().unwrap())];
        let dcval_offset: [u32; 3] = [u32::from_be_bytes(buf[0x28..0x2C].try_into().unwrap()), u32::from_be_bytes(buf[0x2C..0x30].try_into().unwrap()), u32::from_be_bytes(buf[0x30..0x34].try_into().unwrap())];

        HVQM2Frame {
            basisnum_offset: basisnum_offset,
            basnumrn_offset: basnumrn_offset,
            scale_offset: scale_offset,
            fixvl_offset: fixvl_offset,
            dcval_offset: dcval_offset,
        }
    }
}

/*
 * HVQM2KeyFrame : Key frame header (Follows the video header)
 */
pub struct HVQM2KeyFrame {
    /* 0x0 */ pub dcrun_offset: [u32; 3],    /* DC value cold run (0:Y, 1:U, 2:V) */
    /* 0xC */ pub nest_start_x: u16,        /* Base start position (x coordinate) */
    /* 0xE */ pub nest_start_y: u16,        /* Base start position (y coordinate) */
}

impl HVQM2KeyFrame {
    pub fn new(buf: &[u8]) -> HVQM2KeyFrame {
        let dcrun_offset: [u32; 3] = [u32::from_be_bytes(buf[0x00..0x04].try_into().unwrap()), u32::from_be_bytes(buf[0x04..0x08].try_into().unwrap()), u32::from_be_bytes(buf[0x08..0x0C].try_into().unwrap())];
        let nest_start_x: u16 = u16::from_be_bytes(buf[0x0C..0x0E].try_into().unwrap());
        let nest_start_y: u16 = u16::from_be_bytes(buf[0x0E..0x10].try_into().unwrap());

        HVQM2KeyFrame {
            dcrun_offset: dcrun_offset,
            nest_start_x: nest_start_x,
            nest_start_y: nest_start_y,
        }
    }
}

/*
* HVQM2PredictFrame : Predict frame header (Follows video header)
*/
pub struct HVQM2PredictFrame {
    /* 0x0 */ pub movevector_offset: u32,    /* Movement vector */
    /* 0x4 */ pub macroblock_offset: u32,    /* Macro block state flag */
}

impl HVQM2PredictFrame {
    pub fn new(buf: &[u8]) -> HVQM2PredictFrame {
        let movevector_offset: u32 = u32::from_be_bytes(buf[0x00..0x04].try_into().unwrap());
        let macroblock_offset: u32 = u32::from_be_bytes(buf[0x04..0x08].try_into().unwrap());

        HVQM2PredictFrame {
            movevector_offset: movevector_offset,
            macroblock_offset: macroblock_offset,
        }
    }
}
