use std::io::{ Seek, Read, SeekFrom };
use std::vec::IntoIter;
use lewton::inside_ogg::OggStreamReader;
use std::mem;

use crate::SoundSource;

pub struct OggDecoder<T: Seek + Read + Send + 'static> {
    reader: OggStreamReader<T>,
    buffer: IntoIter<i16>,
    done: bool,
}
impl<T: Seek + Read + Send + 'static> OggDecoder<T> {
    pub fn new(data: T) -> Self {
        let mut reader = OggStreamReader::new(data).unwrap();
        // The first packed is always empty
        let _ = reader.read_dec_packet_itl().unwrap();
        Self {
            buffer: reader.read_dec_packet_itl().unwrap().unwrap_or(vec![]).into_iter(),
            reader,
            done: false,
        }
    }
}
impl<T: Seek + Read + Send + 'static> SoundSource for OggDecoder<T> {
    fn channels(&self) -> u16 {
        self.reader.ident_hdr.audio_channels as u16
    }

    fn sample_rate(&self) -> u32 {
        self.reader.ident_hdr.audio_sample_rate
    }

    fn reset(&mut self) {
        unsafe {
            let reader =  mem::replace(&mut self.reader, mem::zeroed());
            let mut source = reader
                .into_inner()
                .into_inner();
            source.seek(SeekFrom::Start(0)).unwrap();
            let reader = OggStreamReader::new(source).unwrap();
            mem::replace(&mut self.reader, reader);
        };
        self.done = false;
        // The first packed is always empty
        let _ = self.reader.read_dec_packet_itl().unwrap();
        self.buffer = self.reader.read_dec_packet_itl().unwrap().unwrap_or(vec![]).into_iter();
    }
    fn write_samples(&mut self, buffer: &mut [i16]) -> usize {
        let mut i = 0;

        'main: while i < buffer.len() {
            if let Some(next) = self.buffer.next() {
                buffer[i] = next;
                i += 1;
            } else {
                while let Some(pck) = self.reader.read_dec_packet_itl().unwrap() {
                    if pck.len() > 0 {
                        self.buffer = pck.into_iter();
                        continue 'main;
                    }
                }
                return i;
            }
        }

        buffer.len()
    }
}