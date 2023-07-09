
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ADPCMFormat {
    Reset,
    Continue,
}

impl ADPCMFormat {
    pub fn new(format: u32) -> Result<ADPCMFormat, ()> {
        match format {
            0 => Ok(ADPCMFormat::Reset),
            1 => Ok(ADPCMFormat::Continue),
            _ => Err(()),
        }
    }
}

/* ADPCM state information structure */
pub struct ADPCMstate {
    // inPtr: &[u8],
    // outPtr: &mut [i16],
    previous: i16,
    hi_nibble: bool, // u8,
    step_index: u8,
}

const D_0001D0: &'static [i32] = &[
    -1,
    -1,
    -1,
    -1,
    2,
    4,
    6,
    8,
    -1,
    -1,
    -1,
    -1,
    2,
    4,
    6,
    8,
];

const D_000210: &'static [i32] = &[
    0x0007,
    0x0008,
    0x0009,
    0x000A,
    0x000B,
    0x000C,
    0x000D,
    0x000E,
    0x0010,
    0x0011,
    0x0013,
    0x0015,
    0x0017,
    0x0019,
    0x001C,
    0x001F,
    0x0022,
    0x0025,
    0x0029,
    0x002D,
    0x0032,
    0x0037,
    0x003C,
    0x0042,
    0x0049,
    0x0050,
    0x0058,
    0x0061,
    0x006B,
    0x0076,
    0x0082,
    0x008F,
    0x009D,
    0x00AD,
    0x00BE,
    0x00D1,
    0x00E6,
    0x00FD,
    0x0117,
    0x0133,
    0x0151,
    0x0173,
    0x0198,
    0x01C1,
    0x01EE,
    0x0220,
    0x0256,
    0x0292,
    0x02D4,
    0x031C,
    0x036C,
    0x03C3,
    0x0424,
    0x048E,
    0x0502,
    0x0583,
    0x0610,
    0x06AB,
    0x0756,
    0x0812,
    0x08E0,
    0x09C3,
    0x0ABD,
    0x0BD0,
    0x0CFF,
    0x0E4C,
    0x0FBA,
    0x114C,
    0x1307,
    0x14EE,
    0x1706,
    0x1954,
    0x1BDC,
    0x1EA5,
    0x21B6,
    0x2515,
    0x28CA,
    0x2CDF,
    0x315B,
    0x364B,
    0x3BB9,
    0x41B2,
    0x4844,
    0x4F7E,
    0x5771,
    0x602F,
    0x69CE,
    0x7462,
    0x7FFF,
];


impl ADPCMstate {
    pub fn new() -> ADPCMstate {
        ADPCMstate {
            previous: 0,
            hi_nibble: false,
            step_index: 0,
        }
    }

    pub fn adpcmDecode(&mut self, instream: &[u8], format: ADPCMFormat, samples: u32, outstream: &mut [i16], ex_stereo: bool) -> () {
        let mut var_t0: i32;
        let mut stepIndex: i32;
        let mut hiNibble: bool; // i32;
        // u8* streamP;
        let mut inOffset = 0;
        let mut outOffset = 0;
        let mut samples_left = samples;

        //if (outstream == NULL) {
        //    outstream = self.outPtr;
        //}

        //if (instream != NULL) {
            // streamP = instream;
            if format == ADPCMFormat::Reset {
                let temp_a0 = instream[inOffset];
                inOffset += 1;
                let t = instream[inOffset];
                inOffset += 1;

                self.previous = (((temp_a0 as u32) << 8) | ((t as u32) & 0x80)) as i16;
                self.step_index = t & 0x7F;
                outstream[outOffset] =  self.previous;
                outOffset += 1;
                if ex_stereo {
                    outstream[outOffset] = self.previous;
                    outOffset += 1;
                }
                samples_left -= 1;
            }

            hiNibble = true;
        //} else {
        //    streamP = self.inPtr;
        //    hiNibble = self.hi_nibble;
        //}

        var_t0 = self.previous as i32;
        stepIndex = self.step_index as i32;
        while samples_left > 0 {
            let mut var_a1: u32;
            let mut temp_a0_2: i32;
            let mut var_v1: i32;

            if hiNibble {
                var_a1 = (instream[inOffset] >> 4) as u32;
            } else {
                var_a1 = (instream[inOffset] & 0xF) as u32;
                inOffset += 1;
            }

            temp_a0_2 = D_000210[stepIndex as usize];
            var_v1 = temp_a0_2 >> 3;
            if (var_a1 & 1) != 0 {
                var_v1 += temp_a0_2 >> 2;
            }
            if (var_a1 & 2) != 0 {
                var_v1 += temp_a0_2 >> 1;
            }
            if (var_a1 & 4) != 0 {
                var_v1 += temp_a0_2;
            }
            if (var_a1 & 8) != 0 {
                var_v1 = -var_v1;
            }

            var_t0 += var_v1;
            if (var_t0 > 0x7FFF) {
                var_t0 = 0x7FFF;
            } else if (var_t0 < -0x8000) {
                var_t0 = -0x8000;
            }

            stepIndex += D_0001D0[var_a1 as usize];
            if stepIndex < 0 {
                stepIndex = 0;
            } else if stepIndex >= D_000210.len() as i32 {
                stepIndex = (D_000210.len() as i32) - 1;
            }

            outstream[outOffset] = var_t0 as i16;
            outOffset += 1;
            if ex_stereo {
                outstream[outOffset] = var_t0 as i16;
                outOffset += 1;
            }

            samples_left -= 1;
            hiNibble = !hiNibble;
        }

        //self.inPtr = streamP;
        self.hi_nibble = hiNibble;
        //self.outPtr = outstream;
        self.previous = var_t0 as i16;
        self.step_index = stepIndex as u8;
    }

}
