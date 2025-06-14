use std::fs::File;
use std::path::Path;
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::{FormatOptions, FormatReader, Track};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::default::{get_codecs, get_probe};

pub fn open_decoder(path: &Path) -> anyhow::Result<(
    Box<dyn FormatReader>,
    Box<dyn Decoder>,
    Track
)> {
    let file = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let probed = get_probe().format(
        &Default::default(),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let track = probed
        .format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow::anyhow!("No supported audio track found"))?
        .clone();

    let decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    let format = probed.format;

    Ok((format, decoder, track.clone()))
}
