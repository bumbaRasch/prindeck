use std::fmt;
use std::time::Duration;

use quick_xml::XmlVersion;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::Url;
use serde::{Deserialize, Serialize};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const NEXT_DOCUMENT_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_BODY_SNIPPET: usize = 200;

const PWG_NS: &str = "http://www.pwg.org/schemas/2010/12/sm";
const SCAN_NS: &str = "http://schemas.hp.com/imaging/escl/2011/05/03";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorMode {
    BlackAndWhite1,
    Grayscale8,
    RGB24,
}

impl ColorMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ColorMode::BlackAndWhite1 => "BlackAndWhite1",
            ColorMode::Grayscale8 => "Grayscale8",
            ColorMode::RGB24 => "RGB24",
        }
    }

    fn from_xml(value: &str) -> Option<Self> {
        match value {
            "BlackAndWhite1" => Some(ColorMode::BlackAndWhite1),
            "Grayscale8" => Some(ColorMode::Grayscale8),
            "RGB24" => Some(ColorMode::RGB24),
            _ => None,
        }
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocFormat {
    #[serde(rename = "application/pdf")]
    Pdf,
    #[serde(rename = "image/jpeg")]
    Jpeg,
}

impl DocFormat {
    pub fn mime(self) -> &'static str {
        match self {
            DocFormat::Pdf => "application/pdf",
            DocFormat::Jpeg => "image/jpeg",
        }
    }

    fn from_xml(value: &str) -> Option<Self> {
        match value {
            "application/pdf" => Some(DocFormat::Pdf),
            "image/jpeg" => Some(DocFormat::Jpeg),
            _ => None,
        }
    }
}

impl fmt::Display for DocFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.mime())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    Platen,
    Feeder,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Platen => "Platen",
            Source::Feeder => "Feeder",
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScannerState {
    Idle,
    Processing,
    Stopped,
}

impl ScannerState {
    fn from_xml(value: &str) -> Option<Self> {
        match value {
            "Idle" => Some(ScannerState::Idle),
            "Processing" => Some(ScannerState::Processing),
            "Stopped" => Some(ScannerState::Stopped),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ScannerState::Idle => "Idle",
            ScannerState::Processing => "Processing",
            ScannerState::Stopped => "Stopped",
        }
    }
}

impl fmt::Display for ScannerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Pending,
    Processing,
    Completed,
    Canceled,
    Aborted,
}

impl JobState {
    fn from_xml(value: &str) -> Option<Self> {
        match value {
            "Pending" => Some(JobState::Pending),
            "Processing" => Some(JobState::Processing),
            "Completed" => Some(JobState::Completed),
            "Canceled" => Some(JobState::Canceled),
            "Aborted" => Some(JobState::Aborted),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            JobState::Pending => "Pending",
            JobState::Processing => "Processing",
            JobState::Completed => "Completed",
            JobState::Canceled => "Canceled",
            JobState::Aborted => "Aborted",
        }
    }
}

impl fmt::Display for JobState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputCaps {
    pub max_width: u32,
    pub max_height: u32,
    pub resolutions: Vec<u32>,
    pub color_modes: Vec<ColorMode>,
    pub formats: Vec<DocFormat>,
}

impl InputCaps {
    fn empty() -> Self {
        InputCaps {
            max_width: 0,
            max_height: 0,
            resolutions: Vec::new(),
            color_modes: Vec::new(),
            formats: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanCaps {
    pub make_and_model: String,
    pub platen: InputCaps,
    pub adf: Option<InputCaps>,
    pub feeder_capacity: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanParams {
    pub source: Source,
    pub dpi: u32,
    pub color: ColorMode,
    pub format: DocFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    pub uri: String,
    pub state: JobState,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannerStatus {
    pub state: ScannerState,
    pub adf_state: Option<String>,
    pub jobs: Vec<JobInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x_offset: u32,
    pub y_offset: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn platen_a4() -> Self {
        Region {
            x_offset: 0,
            y_offset: 0,
            width: 2550,
            height: 3507,
        }
    }

    pub fn from_caps(caps: &InputCaps) -> Self {
        Region {
            x_offset: 0,
            y_offset: 0,
            width: caps.max_width,
            height: caps.max_height,
        }
    }
}

fn region_for(source: Source, input_caps: &InputCaps, platen: &InputCaps) -> Region {
    match source {
        Source::Platen => Region::from_caps(input_caps),
        Source::Feeder => Region {
            height: platen.max_height,
            ..Region::from_caps(input_caps)
        },
    }
}

const FEEDER_PAGE_SECONDS: u64 = 8;
const FEEDER_FETCH_SLACK: Duration = Duration::from_secs(20);

pub fn fetch_timeout(source: Source, caps: &ScanCaps) -> Duration {
    match source {
        Source::Platen => NEXT_DOCUMENT_TIMEOUT,
        Source::Feeder => caps
            .feeder_capacity
            .map_or(NEXT_DOCUMENT_TIMEOUT, |capacity| {
                Duration::from_secs(FEEDER_PAGE_SECONDS) * capacity + FEEDER_FETCH_SLACK
            }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NextDoc {
    Document(Vec<u8>),
    End,
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelOutcome {
    Canceled,
    AlreadyFinished,
}

#[derive(Debug)]
pub enum EsclError {
    Transport(String),
    Timeout,
    Http { status: u16, body: String },
    Parse(String),
    InvalidParams(String),
    InvalidUrl(String),
}

impl fmt::Display for EsclError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EsclError::Transport(message) => write!(f, "scanner connection error: {message}"),
            EsclError::Timeout => write!(f, "the scanner did not respond in time"),
            EsclError::Http { status, body } => {
                if body.is_empty() {
                    write!(f, "the scanner returned HTTP {status}")
                } else {
                    write!(f, "the scanner returned HTTP {status}: {body}")
                }
            }
            EsclError::Parse(message) => {
                write!(f, "failed to parse the scanner response: {message}")
            }
            EsclError::InvalidParams(message) => {
                write!(f, "invalid scan parameters: {message}")
            }
            EsclError::InvalidUrl(message) => write!(f, "invalid request URL: {message}"),
        }
    }
}

impl std::error::Error for EsclError {}

impl ScanParams {
    pub fn to_xml(&self, region: Region) -> String {
        format!(
            "<scan:ScanSettings xmlns:pwg=\"{PWG_NS}\" xmlns:scan=\"{SCAN_NS}\">\
<pwg:Version>2.0</pwg:Version>\
<scan:Intent>Document</scan:Intent>\
<pwg:ScanRegions><pwg:ScanRegion>\
<pwg:ContentRegionUnits>escl:ThreeHundredthsOfInches</pwg:ContentRegionUnits>\
<pwg:XOffset>{}</pwg:XOffset>\
<pwg:YOffset>{}</pwg:YOffset>\
<pwg:Width>{}</pwg:Width>\
<pwg:Height>{}</pwg:Height>\
</pwg:ScanRegion></pwg:ScanRegions>\
<pwg:InputSource>{}</pwg:InputSource>\
<scan:ColorMode>{}</scan:ColorMode>\
<pwg:DocumentFormat>{}</pwg:DocumentFormat>\
<scan:DocumentFormatExt>{}</scan:DocumentFormatExt>\
<scan:XResolution>{dpi}</scan:XResolution>\
<scan:YResolution>{dpi}</scan:YResolution>\
</scan:ScanSettings>",
            region.x_offset,
            region.y_offset,
            region.width,
            region.height,
            self.source.as_str(),
            self.color.as_str(),
            self.format.mime(),
            self.format.mime(),
            dpi = self.dpi,
        )
    }

    pub fn validate(&self, caps: &ScanCaps) -> Result<(), EsclError> {
        let input_caps = match self.source {
            Source::Platen => &caps.platen,
            Source::Feeder => caps.adf.as_ref().ok_or_else(|| {
                EsclError::InvalidParams("the device does not support the feeder".to_string())
            })?,
        };

        if !input_caps.resolutions.contains(&self.dpi) {
            return Err(EsclError::InvalidParams(format!(
                "the device does not support {} dpi",
                self.dpi
            )));
        }

        if !input_caps.color_modes.contains(&self.color) {
            return Err(EsclError::InvalidParams(format!(
                "the device does not support the {} color mode",
                self.color
            )));
        }

        if !input_caps.formats.contains(&self.format) {
            return Err(EsclError::InvalidParams(format!(
                "the device does not support the {} format",
                self.format
            )));
        }

        if self.color == ColorMode::BlackAndWhite1 && self.format == DocFormat::Jpeg {
            return Err(EsclError::InvalidParams(
                "the black-and-white mode is available only for PDF, not JPEG".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct EsclClient {
    base: Url,
    http: reqwest::Client,
}

impl EsclClient {
    pub fn new(mut base: Url) -> Result<Self, EsclError> {
        if !matches!(base.scheme(), "http" | "https") {
            return Err(EsclError::InvalidUrl(format!(
                "only http/https is supported, got {}",
                base.scheme()
            )));
        }

        if base.host_str().is_none() {
            return Err(EsclError::InvalidUrl(
                "the scanner URL has no host".to_string(),
            ));
        }

        if !base.path().ends_with('/') {
            let path = format!("{}/", base.path());
            base.set_path(&path);
        }

        let http = reqwest::Client::builder()
            .connect_timeout(REQUEST_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| EsclError::Transport(error.to_string()))?;

        Ok(EsclClient { base, http })
    }

    pub fn base(&self) -> &Url {
        &self.base
    }

    pub async fn capabilities(&self) -> Result<ScanCaps, EsclError> {
        let xml = self.get_xml("ScannerCapabilities").await?;
        parse_capabilities(&xml)
    }

    pub async fn status(&self) -> Result<ScannerStatus, EsclError> {
        let xml = self.get_xml("ScannerStatus").await?;
        parse_status(&xml)
    }

    pub async fn start(&self, params: &ScanParams) -> Result<Url, EsclError> {
        let caps = self.capabilities().await?;
        params.validate(&caps)?;

        let input_caps = match params.source {
            Source::Platen => &caps.platen,
            Source::Feeder => caps.adf.as_ref().ok_or_else(|| {
                EsclError::InvalidParams("the device does not support the feeder".to_string())
            })?,
        };
        let region = region_for(params.source, input_caps, &caps.platen);
        let body = params.to_xml(region);
        let url = self.endpoint("ScanJobs")?;

        let response = self
            .http
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "text/xml")
            .body(body)
            .send()
            .await
            .map_err(map_transport)?;

        let status = response.status().as_u16();
        if status != 201 {
            let body = response.bytes().await.map_err(map_transport)?;
            return Err(http_error(status, &body));
        }

        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .ok_or_else(|| {
                EsclError::Parse("a 201 response without a Location header".to_string())
            })?
            .to_str()
            .map_err(|error| EsclError::Parse(format!("Location is not a string: {error}")))?;

        resolve_job_url(&self.base, location)
    }

    pub async fn next_document(&self, job: &Url, timeout: Duration) -> Result<NextDoc, EsclError> {
        let url = subresource(job, "NextDocument")?;

        let response = self
            .http
            .get(url)
            .timeout(timeout)
            .send()
            .await
            .map_err(map_transport)?;

        let status = response.status().as_u16();
        let body = response.bytes().await.map_err(map_transport)?;

        classify_next_document(status, body.to_vec())
    }

    pub async fn cancel(&self, job: &Url) -> Result<CancelOutcome, EsclError> {
        let response = self
            .http
            .delete(job.clone())
            .send()
            .await
            .map_err(map_transport)?;

        let status = response.status().as_u16();
        let body = response.bytes().await.map_err(map_transport)?;

        classify_cancel(status, &body)
    }

    async fn get_xml(&self, resource: &str) -> Result<String, EsclError> {
        let url = self.endpoint(resource)?;

        let response = self.http.get(url).send().await.map_err(map_transport)?;
        let status = response.status().as_u16();
        let body = response.bytes().await.map_err(map_transport)?;

        if status != 200 {
            return Err(http_error(status, &body));
        }

        String::from_utf8(body.to_vec())
            .map_err(|error| EsclError::Parse(format!("the response is not UTF-8: {error}")))
    }

    fn endpoint(&self, resource: &str) -> Result<Url, EsclError> {
        self.base
            .join(resource)
            .map_err(|error| EsclError::InvalidUrl(format!("{resource}: {error}")))
    }
}

fn subresource(job: &Url, resource: &str) -> Result<Url, EsclError> {
    let url = format!("{}/{}", job.as_str().trim_end_matches('/'), resource);
    Url::parse(&url).map_err(|error| EsclError::InvalidUrl(format!("{url}: {error}")))
}

fn resolve_job_url(base: &Url, location: &str) -> Result<Url, EsclError> {
    let location = location.trim();

    if location.is_empty() {
        return Err(EsclError::InvalidUrl("empty Location header".to_string()));
    }

    if let Ok(url) = Url::parse(location) {
        return Ok(url);
    }

    base.join(location)
        .map_err(|error| EsclError::InvalidUrl(format!("Location {location}: {error}")))
}

fn map_transport(error: reqwest::Error) -> EsclError {
    if error.is_timeout() {
        EsclError::Timeout
    } else {
        EsclError::Transport(error.to_string())
    }
}

fn http_error(status: u16, body: &[u8]) -> EsclError {
    EsclError::Http {
        status,
        body: body_snippet(body),
    }
}

fn body_snippet(body: &[u8]) -> String {
    let text = String::from_utf8_lossy(body);
    let text = text.trim();

    let mut end = text.len().min(MAX_BODY_SNIPPET);
    while !text.is_char_boundary(end) {
        end -= 1;
    }

    text[..end].to_string()
}

fn classify_next_document(status: u16, body: Vec<u8>) -> Result<NextDoc, EsclError> {
    match status {
        200 => Ok(NextDoc::Document(body)),
        404 => Ok(NextDoc::End),
        410 => Ok(NextDoc::Canceled),
        _ => Err(http_error(status, &body)),
    }
}

fn classify_cancel(status: u16, body: &[u8]) -> Result<CancelOutcome, EsclError> {
    match status {
        200 => Ok(CancelOutcome::Canceled),
        404 => Ok(CancelOutcome::AlreadyFinished),
        _ => Err(http_error(status, body)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapsSection {
    None,
    Platen,
    Adf,
}

#[derive(Debug, Clone, Copy)]
enum CapsField {
    MakeAndModel,
    MaxWidth,
    MaxHeight,
    ColorMode,
    DocumentFormat,
    DocumentFormatExt,
    XResolution,
    FeederCapacity,
}

fn parse_capabilities(xml: &str) -> Result<ScanCaps, EsclError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut make_and_model: Option<String> = None;
    let mut platen: Option<InputCaps> = None;
    let mut adf: Option<InputCaps> = None;
    let mut feeder_capacity: Option<u32> = None;
    let mut section = CapsSection::None;
    let mut current: Option<InputCaps> = None;
    let mut field: Option<CapsField> = None;

    loop {
        let event = reader
            .read_event()
            .map_err(|error| EsclError::Parse(format!("ScannerCapabilities: {error}")))?;

        match event {
            Event::Start(element) => match element.local_name().as_ref() {
                "PlatenInputCaps" => {
                    section = CapsSection::Platen;
                    current = Some(InputCaps::empty());
                }
                "AdfSimplexInputCaps" | "AdfDuplexInputCaps" => {
                    section = CapsSection::Adf;
                    current = Some(InputCaps::empty());
                }
                "MakeAndModel" => field = Some(CapsField::MakeAndModel),
                "MaxWidth" => field = Some(CapsField::MaxWidth),
                "MaxHeight" => field = Some(CapsField::MaxHeight),
                "ColorMode" => field = Some(CapsField::ColorMode),
                "DocumentFormat" => field = Some(CapsField::DocumentFormat),
                "DocumentFormatExt" => field = Some(CapsField::DocumentFormatExt),
                "FeederCapacity" => field = Some(CapsField::FeederCapacity),
                "XResolution" if current.is_some() => field = Some(CapsField::XResolution),
                _ => {}
            },
            Event::Text(text) => {
                let Some(field) = field else {
                    continue;
                };

                let value = text.xml_content(XmlVersion::Implicit1_0);
                let value = value.trim();
                if value.is_empty() {
                    continue;
                }

                match field {
                    CapsField::MakeAndModel if section == CapsSection::None => {
                        make_and_model = Some(value.to_string());
                    }
                    CapsField::MaxWidth => {
                        if let Some(caps) = current.as_mut() {
                            caps.max_width = parse_u32(value, "MaxWidth")?;
                        }
                    }
                    CapsField::MaxHeight => {
                        if let Some(caps) = current.as_mut() {
                            caps.max_height = parse_u32(value, "MaxHeight")?;
                        }
                    }
                    CapsField::ColorMode => {
                        if let Some(caps) = current.as_mut()
                            && let Some(mode) = ColorMode::from_xml(value)
                        {
                            push_unique(&mut caps.color_modes, mode);
                        }
                    }
                    CapsField::DocumentFormat | CapsField::DocumentFormatExt => {
                        if let Some(caps) = current.as_mut()
                            && let Some(format) = DocFormat::from_xml(value)
                        {
                            push_unique(&mut caps.formats, format);
                        }
                    }
                    CapsField::XResolution => {
                        if let Some(caps) = current.as_mut() {
                            let resolution = parse_u32(value, "XResolution")?;
                            push_unique(&mut caps.resolutions, resolution);
                        }
                    }
                    CapsField::FeederCapacity => {
                        feeder_capacity = Some(parse_u32(value, "FeederCapacity")?);
                    }
                    _ => {}
                }
            }
            Event::End(element) => {
                field = None;

                match element.local_name().as_ref() {
                    "PlatenInputCaps" => {
                        if let Some(caps) = current.take() {
                            platen = Some(caps);
                        }
                        section = CapsSection::None;
                    }
                    "AdfSimplexInputCaps" | "AdfDuplexInputCaps" => {
                        if let Some(caps) = current.take() {
                            adf = Some(caps);
                        }
                        section = CapsSection::None;
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    let make_and_model = make_and_model
        .ok_or_else(|| EsclError::Parse("no MakeAndModel in capabilities".to_string()))?;
    let platen =
        platen.ok_or_else(|| EsclError::Parse("no PlatenInputCaps in capabilities".to_string()))?;

    Ok(ScanCaps {
        make_and_model,
        platen,
        adf,
        feeder_capacity,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusField {
    ScannerState,
    AdfState,
    JobUri,
    JobState,
    JobStateReason,
}

#[derive(Default)]
struct JobBuilder {
    uri: Option<String>,
    state: Option<JobState>,
    reasons: Vec<String>,
}

fn parse_status(xml: &str) -> Result<ScannerStatus, EsclError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut state: Option<ScannerState> = None;
    let mut adf_state: Option<String> = None;
    let mut jobs: Vec<JobInfo> = Vec::new();
    let mut job: Option<JobBuilder> = None;
    let mut field: Option<StatusField> = None;

    loop {
        let event = reader
            .read_event()
            .map_err(|error| EsclError::Parse(format!("ScannerStatus: {error}")))?;

        match event {
            Event::Start(element) => match element.local_name().as_ref() {
                "State" if job.is_none() => field = Some(StatusField::ScannerState),
                "AdfState" => field = Some(StatusField::AdfState),
                "JobInfo" => job = Some(JobBuilder::default()),
                "JobUri" if job.is_some() => field = Some(StatusField::JobUri),
                "JobState" if job.is_some() => field = Some(StatusField::JobState),
                "JobStateReason" if job.is_some() => field = Some(StatusField::JobStateReason),
                _ => {}
            },
            Event::Text(text) => {
                let Some(field) = field else {
                    continue;
                };

                let value = text.xml_content(XmlVersion::Implicit1_0);
                let value = value.trim();
                if value.is_empty() {
                    continue;
                }

                match field {
                    StatusField::ScannerState => {
                        if let Some(parsed) = ScannerState::from_xml(value) {
                            state = Some(parsed);
                        }
                    }
                    StatusField::AdfState => adf_state = Some(value.to_string()),
                    StatusField::JobUri => {
                        if let Some(job) = job.as_mut() {
                            job.uri = Some(value.to_string());
                        }
                    }
                    StatusField::JobState => {
                        if let Some(job) = job.as_mut() {
                            job.state = JobState::from_xml(value);
                        }
                    }
                    StatusField::JobStateReason => {
                        if let Some(job) = job.as_mut() {
                            job.reasons.push(value.to_string());
                        }
                    }
                }
            }
            Event::End(element) => {
                field = None;

                if element.local_name().as_ref() == "JobInfo"
                    && let Some(job) = job.take()
                {
                    match (job.uri, job.state) {
                        (Some(uri), Some(state)) => jobs.push(JobInfo {
                            uri,
                            state,
                            reasons: job.reasons,
                        }),
                        _ => tracing::debug!("skipped an incomplete JobInfo entry"),
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    let state = state.ok_or_else(|| EsclError::Parse("no pwg:State in the status".to_string()))?;

    Ok(ScannerStatus {
        state,
        adf_state,
        jobs,
    })
}

fn parse_u32(value: &str, field: &str) -> Result<u32, EsclError> {
    value
        .parse::<u32>()
        .map_err(|error| EsclError::Parse(format!("{field}: not a number ({value}): {error}")))
}

fn push_unique<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAPABILITIES_XML: &str = include_str!("../../fixtures/escl/capabilities.xml");
    const STATUS_XML: &str = include_str!("../../fixtures/escl/status.xml");
    const STATUS_ACTIVE_XML: &str = include_str!("../../fixtures/escl/status-job-active.xml");
    const SCAN_SETTINGS_XML: &str =
        include_str!("../../fixtures/escl/scansettings-platen-300-gray-pdf.xml");

    fn sample_params() -> ScanParams {
        ScanParams {
            source: Source::Platen,
            dpi: 300,
            color: ColorMode::Grayscale8,
            format: DocFormat::Pdf,
        }
    }

    #[test]
    fn parses_capabilities() {
        let caps = parse_capabilities(CAPABILITIES_XML).expect("capabilities parse");

        assert_eq!(caps.make_and_model, "Brother MFC-L2710DW series");

        assert_eq!(caps.platen.max_width, 2550);
        assert_eq!(caps.platen.max_height, 3507);
        assert_eq!(caps.platen.resolutions, vec![100, 200, 300, 600]);
        assert_eq!(
            caps.platen.color_modes,
            vec![
                ColorMode::BlackAndWhite1,
                ColorMode::Grayscale8,
                ColorMode::RGB24
            ]
        );
        assert_eq!(caps.platen.formats, vec![DocFormat::Pdf, DocFormat::Jpeg]);

        let adf = caps.adf.expect("ADF is present");
        assert_eq!(adf.max_width, 2550);
        assert_eq!(adf.max_height, 4200);
        assert_eq!(caps.feeder_capacity, Some(20));
        assert_eq!(adf.resolutions, vec![100, 200, 300, 600]);
        assert_eq!(
            adf.color_modes,
            vec![
                ColorMode::BlackAndWhite1,
                ColorMode::Grayscale8,
                ColorMode::RGB24
            ]
        );
        assert_eq!(adf.formats, vec![DocFormat::Pdf, DocFormat::Jpeg]);
    }

    #[test]
    fn feeder_region_clips_to_a4_height() {
        let caps = parse_capabilities(CAPABILITIES_XML).expect("capabilities parse");
        let adf = caps.adf.expect("ADF is present");
        assert_eq!(adf.max_height, 4200);

        let feeder = region_for(Source::Feeder, &adf, &caps.platen);
        assert_eq!(feeder.width, 2550);
        assert_eq!(feeder.height, 3507);

        let platen = region_for(Source::Platen, &caps.platen, &caps.platen);
        assert_eq!(platen.height, 3507);

        let params = ScanParams {
            source: Source::Feeder,
            ..sample_params()
        };
        let xml = params.to_xml(feeder);
        assert!(xml.contains("<pwg:Height>3507</pwg:Height>"));
        assert!(!xml.contains("<pwg:Height>4200</pwg:Height>"));
    }

    #[test]
    fn fetch_timeouts_scale_with_the_stack() {
        let caps = parse_capabilities(CAPABILITIES_XML).expect("capabilities parse");

        assert_eq!(
            fetch_timeout(Source::Platen, &caps),
            Duration::from_secs(120)
        );
        assert_eq!(
            fetch_timeout(Source::Feeder, &caps),
            Duration::from_secs(180)
        );

        let mut single = caps.clone();
        single.feeder_capacity = Some(1);
        assert_eq!(
            fetch_timeout(Source::Feeder, &single),
            Duration::from_secs(28)
        );

        let mut unknown = caps;
        unknown.feeder_capacity = None;
        assert_eq!(
            fetch_timeout(Source::Feeder, &unknown),
            Duration::from_secs(120)
        );
    }

    #[test]
    fn scan_settings_are_byte_identical_to_fixture() {
        let xml = sample_params().to_xml(Region::platen_a4());

        assert_eq!(xml, SCAN_SETTINGS_XML);
        assert!(!xml.contains('\n'));
        assert!(!xml.contains('\r'));
        assert!(!xml.contains("<?xml"));
    }

    #[test]
    fn validate_rejects_unsupported_dpi() {
        let caps = parse_capabilities(CAPABILITIES_XML).expect("capabilities parse");
        let params = ScanParams {
            dpi: 150,
            ..sample_params()
        };

        assert!(matches!(
            params.validate(&caps),
            Err(EsclError::InvalidParams(_))
        ));
    }

    #[test]
    fn validate_rejects_black_and_white_jpeg() {
        let caps = parse_capabilities(CAPABILITIES_XML).expect("capabilities parse");
        let params = ScanParams {
            color: ColorMode::BlackAndWhite1,
            format: DocFormat::Jpeg,
            ..sample_params()
        };

        assert!(matches!(
            params.validate(&caps),
            Err(EsclError::InvalidParams(_))
        ));
    }

    #[test]
    fn validate_accepts_supported_combination() {
        let caps = parse_capabilities(CAPABILITIES_XML).expect("capabilities parse");
        let params = ScanParams {
            color: ColorMode::RGB24,
            ..sample_params()
        };

        assert!(params.validate(&caps).is_ok());
    }

    #[test]
    fn validate_rejects_feeder_without_adf() {
        let mut caps = parse_capabilities(CAPABILITIES_XML).expect("capabilities parse");
        caps.adf = None;
        let params = ScanParams {
            source: Source::Feeder,
            ..sample_params()
        };

        assert!(matches!(
            params.validate(&caps),
            Err(EsclError::InvalidParams(_))
        ));
    }

    #[test]
    fn parses_idle_status() {
        let status = parse_status(STATUS_XML).expect("status parse");

        assert_eq!(status.state, ScannerState::Idle);
        assert_eq!(status.adf_state.as_deref(), Some("ScannerAdfEmpty"));
        assert!(status.jobs.is_empty());
    }

    #[test]
    fn parses_all_jobs_from_active_status() {
        let status = parse_status(STATUS_ACTIVE_XML).expect("status parse");

        assert_eq!(status.state, ScannerState::Processing);
        assert_eq!(status.jobs.len(), 16);

        assert!(status.jobs.iter().any(|job| job.state == JobState::Pending));
        assert!(
            status
                .jobs
                .iter()
                .any(|job| job.state == JobState::Completed)
        );
        assert!(
            status
                .jobs
                .iter()
                .any(|job| job.state == JobState::Canceled)
        );

        for reason in [
            "JobScanning",
            "JobCompletedSuccessfully",
            "JobCanceledAtDevice",
        ] {
            assert!(
                status
                    .jobs
                    .iter()
                    .any(|job| job.reasons.iter().any(|item| item == reason)),
                "reason {reason} must be parsed"
            );
        }
    }

    #[test]
    fn finds_job_by_uri_in_inline_status() {
        let xml = r#"<scan:ScannerStatus xmlns:pwg="http://www.pwg.org/schemas/2010/12/sm" xmlns:scan="http://schemas.hp.com/imaging/escl/2011/05/03"><pwg:State>Processing</pwg:State><scan:Jobs><scan:JobInfo><pwg:JobUri>/eSCL/ScanJobs/first</pwg:JobUri><pwg:JobState>Completed</pwg:JobState><pwg:JobStateReasons><pwg:JobStateReason>JobCompletedSuccessfully</pwg:JobStateReason></pwg:JobStateReasons></scan:JobInfo><scan:JobInfo><pwg:JobUri>/eSCL/ScanJobs/unique-job</pwg:JobUri><pwg:JobState>Pending</pwg:JobState><pwg:JobStateReasons><pwg:JobStateReason>JobScanning</pwg:JobStateReason></pwg:JobStateReasons></scan:JobInfo></scan:Jobs></scan:ScannerStatus>"#;

        let status = parse_status(xml).expect("inline status parse");
        assert_eq!(status.jobs.len(), 2);

        let job = status
            .jobs
            .iter()
            .find(|job| job.uri == "/eSCL/ScanJobs/unique-job")
            .expect("job is present");

        assert_eq!(job.state, JobState::Pending);
        assert_eq!(job.reasons, vec!["JobScanning".to_string()]);
    }

    #[test]
    fn http_errors_carry_status_and_snippet() {
        for status in [409, 500, 503] {
            match http_error(status, b"") {
                EsclError::Http {
                    status: actual,
                    body,
                } => {
                    assert_eq!(actual, status);
                    assert!(body.is_empty());
                }
                other => panic!("expected Http error for {status}, got {other:?}"),
            }
        }
    }

    #[test]
    fn http_error_body_is_truncated_to_200_bytes() {
        let body = vec![b'x'; 500];

        match http_error(500, &body) {
            EsclError::Http { body, .. } => assert!(body.len() <= MAX_BODY_SNIPPET),
            other => panic!("expected Http error, got {other:?}"),
        }
    }

    #[test]
    fn classifies_next_document_statuses() {
        assert_eq!(
            classify_next_document(200, b"pdf".to_vec()).expect("document"),
            NextDoc::Document(b"pdf".to_vec())
        );
        assert_eq!(
            classify_next_document(404, Vec::new()).expect("end"),
            NextDoc::End
        );
        assert_eq!(
            classify_next_document(410, Vec::new()).expect("canceled"),
            NextDoc::Canceled
        );

        assert!(matches!(
            classify_next_document(500, Vec::new()),
            Err(EsclError::Http { status: 500, .. })
        ));
    }

    #[test]
    fn classifies_cancel_statuses() {
        assert_eq!(
            classify_cancel(200, b"").expect("canceled"),
            CancelOutcome::Canceled
        );
        assert_eq!(
            classify_cancel(404, b"").expect("already finished"),
            CancelOutcome::AlreadyFinished
        );

        assert!(matches!(
            classify_cancel(503, b""),
            Err(EsclError::Http { status: 503, .. })
        ));
    }

    #[test]
    fn resolves_absolute_and_relative_location() {
        let base = Url::parse("http://192.168.187.36/eSCL/").expect("base url");

        assert_eq!(
            resolve_job_url(&base, "http://192.168.187.36/eSCL/ScanJobs/abc")
                .expect("absolute")
                .as_str(),
            "http://192.168.187.36/eSCL/ScanJobs/abc"
        );
        assert_eq!(
            resolve_job_url(&base, "/eSCL/ScanJobs/abc")
                .expect("root relative")
                .as_str(),
            "http://192.168.187.36/eSCL/ScanJobs/abc"
        );
        assert_eq!(
            resolve_job_url(&base, "ScanJobs/abc")
                .expect("relative")
                .as_str(),
            "http://192.168.187.36/eSCL/ScanJobs/abc"
        );
    }
}
