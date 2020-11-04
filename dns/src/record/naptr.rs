use log::*;

use crate::strings::{Labels, ReadLabels};
use crate::wire::*;


/// A **NAPTR** _(naming authority pointer)_ record,
#[derive(PartialEq, Debug)]
pub struct NAPTR {

    /// The order in which NAPTR records must be processed.
    pub order: u16,

    /// The DDDS priority.
    pub preference: u16,

    /// A set of characters that control the rewriting and interpretation of
    /// the other fields.
    pub flags: String,

    /// The service parameters applicable to this delegation path.
    pub service: String,

    /// A regular expression that gets applied to a string in order to
    /// construct the next domain name to look up using the DDDS algorithm.
    pub regex: String,

    /// The replacement domain name as part of the DDDS algorithm.
    pub replacement: Labels,
}

impl Wire for NAPTR {
    const NAME: &'static str = "MX";
    const RR_TYPE: u16 = 35;

    #[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate)]
    fn read(stated_length: u16, c: &mut Cursor<&[u8]>) -> Result<Self, WireError> {
        let order = c.read_u16::<BigEndian>()?;
        trace!("Parsed order -> {:?}", order);

        let preference = c.read_u16::<BigEndian>()?;
        trace!("Parsed preference -> {:?}", preference);

        let flags_length = c.read_u8()?;
        trace!("Parsed flags length -> {:?}", flags_length);

        let mut flags_buffer = Vec::with_capacity(flags_length.into());
        for _ in 0 .. flags_length {
            flags_buffer.push(c.read_u8()?);
        }

        let flags = String::from_utf8_lossy(&flags_buffer).to_string();
        trace!("Parsed flags -> {:?}", flags);

        let service_length = c.read_u8()?;
        trace!("Parsed service length -> {:?}", service_length);

        let mut service_buffer = Vec::with_capacity(service_length.into());
        for _ in 0 .. service_length {
            service_buffer.push(c.read_u8()?);
        }

        let service = String::from_utf8_lossy(&service_buffer).to_string();
        trace!("Parsed service -> {:?}", service);

        let regex_length = c.read_u8()?;
        trace!("Parsed regex length -> {:?}", regex_length);

        let mut regex_buffer = Vec::with_capacity(regex_length.into());
        for _ in 0 .. regex_length {
            regex_buffer.push(c.read_u8()?);
        }

        let regex = String::from_utf8_lossy(&regex_buffer).to_string();
        trace!("Parsed regex -> {:?}", regex);

        let (replacement, replacement_length) = c.read_labels()?;
        trace!("Parsed replacement -> {:?}", replacement);

        let length_after_labels = 2 + 2 +
            1 + u16::from(flags_length) + 1 + u16::from(service_length) +
            1 + u16::from(regex_length) + replacement_length;

        if stated_length == length_after_labels {
            Ok(Self { order, preference, flags, service, regex, replacement })
        }
        else {
            Err(WireError::WrongLabelLength { stated_length, length_after_labels })
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parses() {
        let buf = &[
            0x00, 0x05,  // order
            0x00, 0x0a,  // preference
            0x01,  // flags length
            0x73,  // flags
            0x03,  // service length
            0x53, 0x52, 0x56,  // service
            0x0e,  // regex length
            0x5c, 0x64, 0x5c, 0x64, 0x3a, 0x5c, 0x64, 0x5c, 0x64, 0x3a, 0x5c,
            0x64, 0x5c, 0x64,  // regex
            0x0b, 0x73, 0x72, 0x76, 0x2d, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c,
            0x65, 0x06, 0x6c, 0x6f, 0x6f, 0x6b, 0x75, 0x70, 0x03, 0x64, 0x6f,
            0x67, 0x00,  // replacement
        ];

        assert_eq!(NAPTR::read(buf.len() as _, &mut Cursor::new(buf)).unwrap(),
                   NAPTR {
                       order: 5,
                       preference: 10,
                       flags: "s".into(),
                       service: "SRV".into(),
                       regex: "\\d\\d:\\d\\d:\\d\\d".into(),
                       replacement: Labels::encode("srv-example.lookup.dog").unwrap(),
                   });
    }

    #[test]
    fn incorrect_length() {
        let buf = &[
            0x00, 0x05,  // order
            0x00, 0x0a,  // preference
            0x01,  // flags length
            0x73,  // flags
            0x03,  // service length
            0x53, 0x52, 0x56,  // service
            0x01,  // regex length
            0x64,  // regex,
            0x00,  // replacement
        ];

        assert_eq!(NAPTR::read(11, &mut Cursor::new(buf)),
                   Err(WireError::WrongLabelLength { stated_length: 11, length_after_labels: 13 }));
    }

    #[test]
    fn record_empty() {
        assert_eq!(NAPTR::read(0, &mut Cursor::new(&[])),
                   Err(WireError::IO));
    }

    #[test]
    fn buffer_ends_abruptly() {
        let buf = &[
            0x00, 0x0A,  // order
        ];

        assert_eq!(NAPTR::read(23, &mut Cursor::new(buf)),
                   Err(WireError::IO));
    }
}