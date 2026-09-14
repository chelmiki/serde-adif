//! # Serde ADIF
//!
//! ADIF (Amateur Data Interchange Format) is an open standard used to
//! exchange ham radio log data between software and websites. It uses
//! text-based records, for example a QLog export:
//!
//! ```text
//! ### QLog ADIF Export
//! <ADIF_VER:5>3.1.4
//! <PROGRAMID:4>QLog
//! <PROGRAMVERSION:6>0.40.1
//! <CREATED_TIMESTAMP:15>20260417 170249
//! <EOH>
//!
//! <call:7>EI0IRTS
//! <freq:7:N>14.0757
//! <mode:3>FT8
//! <tx_pwr:2>20
//! <qso_date:8:D>20240120
//! <eor>
//!
//! <call:6>EI4JKB
//! <freq:6:N>7.0749
//! <mode:3>FT8
//! <tx_pwr:2>20
//! <qso_date:8:D>20240121
//! <eor>
//! ```
//!
//! # Parsing ADIF into Rust data structures
//!
//! ```
//! use serde_derive::Deserialize;
//! use serde_adif::Adif;
//!
//! #[derive(Deserialize, Debug)]
//! struct Header {
//!     adif_ver: String,
//! }
//!
//! #[derive(Deserialize, Debug)]
//! struct Qso {
//!     call: String,
//!     freq: f32,
//!     mode: String,
//!     tx_pwr: Option<u16>,
//!     qso_date: String,
//! }
//!
//! let data = "\
//! ### QLog ADIF Export
//! <ADIF_VER:5>3.1.4
//! <EOH>
//! <call:7>EI0IRTS<freq:7:N>14.0757<mode:3>FT8<tx_pwr:2>20<qso_date:8:D>20240120<eor>
//! <call:6>EI4JKB<freq:6:N>7.0749<mode:3>FT8<tx_pwr:2>20<qso_date:8:D>20240121<eor>
//! ";
//!
//! let adif = serde_adif::from_str::<Header, Qso>(data).unwrap();
//! assert_eq!(adif.records.len(), 2);
//! assert_eq!(adif.records[0].call, "EI0IRTS");
//! assert_eq!(adif.records[1].tx_pwr, Some(20));
//! ```
//!
//! # Creating ADIF by serializing Rust data structures
//!
//! ```
//! use serde_derive::Serialize;
//! use serde_adif::Adif;
//!
//! #[derive(Serialize)]
//! struct Qso {
//!     call: String,
//!     freq: f32,
//!     mode: String,
//!     tx_pwr: Option<u16>,
//!     qso_date: String,
//! }
//!
//! let qsos = vec![
//!     Qso {
//!         call: "EI0IRTS".to_string(),
//!         freq: 14.0757,
//!         mode: "FT8".to_string(),
//!         tx_pwr: Some(20),
//!         qso_date: "20240120".to_string(),
//!     },
//!     Qso {
//!         call: "EI4JKB".to_string(),
//!         freq: 7.0749,
//!         mode: "FT8".to_string(),
//!         tx_pwr: Some(20),
//!         qso_date: "20240121".to_string(),
//!     },
//! ];
//!
//! let adif = Adif {
//!     header: None,
//!     records: qsos,
//! };
//!
//! let adif_text = serde_adif::to_string::<(), Qso>(&adif).unwrap();
//! assert!(adif_text.contains("<call:7>EI0IRTS"));
//! assert!(adif_text.contains("<call:6>EI4JKB"));
//! ```
//!
//! # Reading and writing ADIF files
//!
//! `from_str`/`to_string` work on data already in memory - reading or
//! writing an `.adi` file on disk is just standard [`std::fs`] usage:
//!
//! ```no_run
//! use serde_derive::{Deserialize, Serialize};
//! use serde_adif::Adif;
//!
//! #[derive(Deserialize, Serialize)]
//! struct Header {};
//!
//! #[derive(Deserialize, Serialize)]
//! struct Qso {
//!     call: String,
//!     freq: f32,
//!     mode: String,
//!     tx_pwr: Option<u16>,
//!     qso_date: String,
//! }
//!
//! // Reading
//! let data = std::fs::read_to_string("log.adi").unwrap();
//! let adif = serde_adif::from_str::<Header, Qso>(&data).unwrap();
//!
//! // Writing
//! let adif_text = serde_adif::to_string(&adif).unwrap();
//! std::fs::write("log.adi", adif_text).unwrap();
//! ```
//!
mod adif;
mod de;
mod error;
mod ser;

pub(crate) const EOR: &str = "<EOR>";
pub(crate) const EOH: &str = "<EOH>";

pub use crate::adif::Adif;
pub use crate::de::from_str;
pub use crate::error::{Error, Result};
pub use crate::ser::to_string;
