use crate::NokhwaError;
use mozjpeg::Decompress;
use std::{
    cmp::Ordering,
    convert::TryFrom,
    fmt::{Display, Formatter},
    slice::from_raw_parts,
};

#[cfg(feature = "input-msmf")]
use nokhwa_bindings_windows::{
    MFCameraFormat, MFControl, MFFrameFormat, MFResolution, MediaFoundationControls,
    MediaFoundationDeviceDescriptor,
};
#[cfg(feature = "input-uvc")]
use uvc::StreamFormat;
#[cfg(feature = "input-v4l")]
use v4l::{control::Description, Format, FourCC};

/// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC` <br>
/// YUYV is a mathematical color space. You can read more [here.](https://en.wikipedia.org/wiki/YCbCr) <br>
/// MJPEG is a motion-jpeg compressed frame, it allows for high frame rates.
#[derive(Copy, Clone, Debug, PartialEq, Hash, PartialOrd, Ord, Eq)]
pub enum FrameFormat {
    MJPEG,
    YUYV,
}
impl Display for FrameFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameFormat::MJPEG => {
                write!(f, "MJPEG")
            }
            FrameFormat::YUYV => {
                write!(f, "YUYV")
            }
        }
    }
}

#[cfg(feature = "input-uvc")]
impl From<FrameFormat> for uvc::FrameFormat {
    fn from(ff: FrameFormat) -> Self {
        match ff {
            FrameFormat::MJPEG => uvc::FrameFormat::MJPEG,
            FrameFormat::YUYV => uvc::FrameFormat::YUYV,
        }
    }
}

#[cfg(feature = "input-msmf")]
impl From<MFFrameFormat> for FrameFormat {
    fn from(mf_ff: MFFrameFormat) -> Self {
        match mf_ff {
            MFFrameFormat::MJPEG => FrameFormat::MJPEG,
            MFFrameFormat::YUYV => FrameFormat::YUYV,
        }
    }
}

impl From<FrameFormat> for MFFrameFormat {
    fn from(ff: FrameFormat) -> Self {
        match ff {
            FrameFormat::MJPEG => MFFrameFormat::MJPEG,
            FrameFormat::YUYV => MFFrameFormat::YUYV,
        }
    }
}

/// Describes a Resolution.
/// This struct consists of a Width and a Height value (x,y). <br>
/// Note: the [`Ord`] implementation of this struct is flipped from highest to lowest.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Resolution {
    pub width_x: u32,
    pub height_y: u32,
}

impl Resolution {
    /// Create a new resolution from 2 image size coordinates.
    #[must_use]
    pub fn new(x: u32, y: u32) -> Self {
        Resolution {
            width_x: x,
            height_y: y,
        }
    }

    /// Get the width of Resolution
    #[must_use]
    pub fn width(self) -> u32 {
        self.width_x
    }

    /// Get the height of Resolution
    #[must_use]
    pub fn height(self) -> u32 {
        self.height_y
    }

    /// Get the x (width) of Resolution
    #[must_use]
    pub fn x(self) -> u32 {
        self.width_x
    }

    /// Get the y (height) of Resolution
    #[must_use]
    pub fn y(self) -> u32 {
        self.height_y
    }
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.x(), self.y())
    }
}

impl PartialOrd for Resolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Resolution {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.x().cmp(&other.x()) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.y().cmp(&other.y()),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

#[cfg(feature = "input-msmf")]
impl From<MFResolution> for Resolution {
    fn from(mf_res: MFResolution) -> Self {
        Resolution {
            width_x: mf_res.width_x,
            height_y: mf_res.height_y,
        }
    }
}

#[cfg(feature = "input-msmf")]
impl From<Resolution> for MFResolution {
    fn from(res: Resolution) -> Self {
        MFResolution {
            width_x: res.width(),
            height_y: res.height(),
        }
    }
}

/// This is a convenience struct that holds all information about the format of a webcam stream.
/// It consists of a [`Resolution`], [`FrameFormat`], and a frame rate(u8).
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub struct CameraFormat {
    resolution: Resolution,
    format: FrameFormat,
    frame_rate: u32,
}

impl CameraFormat {
    /// Construct a new [`CameraFormat`]
    #[must_use]
    pub fn new(resolution: Resolution, format: FrameFormat, framerate: u32) -> Self {
        CameraFormat {
            resolution,
            format,
            frame_rate: framerate,
        }
    }

    /// [`CameraFormat::new()`], but raw.
    #[must_use]
    pub fn new_from(res_x: u32, res_y: u32, format: FrameFormat, fps: u32) -> Self {
        CameraFormat {
            resolution: Resolution {
                width_x: res_x,
                height_y: res_y,
            },
            format,
            frame_rate: fps,
        }
    }

    /// Get the resolution of the current [`CameraFormat`]
    #[must_use]
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Get the width of the resolution of the current [`CameraFormat`]
    #[must_use]
    pub fn width(&self) -> u32 {
        self.resolution.width()
    }

    /// Get the height of the resolution of the current [`CameraFormat`]
    #[must_use]
    pub fn height(&self) -> u32 {
        self.resolution.height()
    }

    /// Set the [`CameraFormat`]'s resolution.
    pub fn set_resolution(&mut self, resolution: Resolution) {
        self.resolution = resolution;
    }

    /// Get the frame rate of the current [`CameraFormat`]
    #[must_use]
    pub fn frame_rate(&self) -> u32 {
        self.frame_rate
    }

    /// Set the [`CameraFormat`]'s frame rate.
    pub fn set_frame_rate(&mut self, frame_rate: u32) {
        self.frame_rate = frame_rate;
    }

    /// Get the [`CameraFormat`]'s format.
    #[must_use]
    pub fn format(&self) -> FrameFormat {
        self.format
    }

    /// Set the [`CameraFormat`]'s format.
    pub fn set_format(&mut self, format: FrameFormat) {
        self.format = format;
    }
}

#[cfg(feature = "input-uvc")]
impl From<CameraFormat> for StreamFormat {
    fn from(cf: CameraFormat) -> Self {
        StreamFormat {
            width: cf.width(),
            height: cf.height(),
            fps: cf.frame_rate(),
            format: cf.format().into(),
        }
    }
}

impl Default for CameraFormat {
    fn default() -> Self {
        CameraFormat {
            resolution: Resolution::new(640, 480),
            format: FrameFormat::MJPEG,
            frame_rate: 15,
        }
    }
}

impl Display for CameraFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}FPS, {} Format",
            self.resolution, self.frame_rate, self.format
        )
    }
}

#[cfg(feature = "input-msmf")]
impl From<MFCameraFormat> for CameraFormat {
    fn from(mf_cam_fmt: MFCameraFormat) -> Self {
        CameraFormat {
            resolution: mf_cam_fmt.resolution().into(),
            format: mf_cam_fmt.format().into(),
            frame_rate: mf_cam_fmt.framerate(),
        }
    }
}

#[cfg(feature = "input-msmf")]
impl From<CameraFormat> for MFCameraFormat {
    fn from(cf: CameraFormat) -> Self {
        MFCameraFormat::new(cf.resolution.into(), cf.format.into(), cf.frame_rate)
    }
}

#[cfg(feature = "input-v4l")]
impl From<CameraFormat> for Format {
    fn from(cam_fmt: CameraFormat) -> Self {
        let pxfmt = match cam_fmt.format() {
            FrameFormat::MJPEG => FourCC::new(b"MJPG"),
            FrameFormat::YUYV => FourCC::new(b"YUYV"),
        };

        Format::new(cam_fmt.width(), cam_fmt.height(), pxfmt)
    }
}

/// Information about a Camera e.g. its name.
/// `description` amd `misc` may contain backend-specific information.
/// `index` is a camera's index given to it by (usually) the OS usually in the order it is known to the system.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct CameraInfo {
    human_name: String,
    description: String,
    misc: String,
    index: usize,
}

impl CameraInfo {
    /// Create a new [`CameraInfo`].
    #[must_use]
    pub fn new(human_name: String, description: String, misc: String, index: usize) -> Self {
        CameraInfo {
            human_name,
            description,
            misc,
            index,
        }
    }

    /// Get a reference to the device info's human name.
    #[must_use]
    pub fn human_name(&self) -> &String {
        &self.human_name
    }

    /// Set the device info's human name.
    pub fn set_human_name(&mut self, human_name: String) {
        self.human_name = human_name;
    }

    /// Get a reference to the device info's description.
    #[must_use]
    pub fn description(&self) -> &String {
        &self.description
    }

    /// Set the device info's description.
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// Get a reference to the device info's misc.
    #[must_use]
    pub fn misc(&self) -> &String {
        &self.misc
    }

    /// Set the device info's misc.
    pub fn set_misc(&mut self, misc: String) {
        self.misc = misc;
    }

    /// Get a reference to the device info's index.
    #[must_use]
    pub fn index(&self) -> &usize {
        &self.index
    }

    /// Set the device info's index.
    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

impl PartialOrd for CameraInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CameraInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl Display for CameraInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, Description: {}, Extra: {}, Index: {}",
            self.human_name, self.description, self.misc, self.index
        )
    }
}

#[cfg(feature = "input-msmf")]
impl From<MediaFoundationDeviceDescriptor<'_>> for CameraInfo {
    fn from(dev_desc: MediaFoundationDeviceDescriptor<'_>) -> Self {
        CameraInfo {
            human_name: dev_desc.name_as_string(),
            description: "Media Foundation Device".to_string(),
            misc: dev_desc.link_as_string(),
            index: dev_desc.index(),
        }
    }
}

/// The list of known camera controls to the library. <br>
/// These can control the picture brightness, etc. <br>
/// Note that not all backends/devices support all these. Run [`supported_camera_controls()`](crate::CaptureBackendTrait::supported_camera_controls) to see which ones can be set.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum KnownCameraControls {
    Brightness,
    Contrast,
    Hue,
    Saturation,
    Sharpness,
    Gamma,
    ColorEnable,
    WhiteBalance,
    BacklightComp,
    Gain,
    Pan,
    Tilt,
    Roll,
    Zoom,
    Exposure,
    Iris,
    Focus,
}

#[must_use]
pub fn all_known_camera_controls() -> [KnownCameraControls; 17] {
    [
        KnownCameraControls::Brightness,
        KnownCameraControls::Contrast,
        KnownCameraControls::Hue,
        KnownCameraControls::Saturation,
        KnownCameraControls::Sharpness,
        KnownCameraControls::Gamma,
        KnownCameraControls::ColorEnable,
        KnownCameraControls::WhiteBalance,
        KnownCameraControls::BacklightComp,
        KnownCameraControls::Gain,
        KnownCameraControls::Pan,
        KnownCameraControls::Tilt,
        KnownCameraControls::Roll,
        KnownCameraControls::Zoom,
        KnownCameraControls::Exposure,
        KnownCameraControls::Iris,
        KnownCameraControls::Focus,
    ]
}

impl Display for KnownCameraControls {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self)
    }
}

#[cfg(feature = "input-msmf")]
impl From<MediaFoundationControls> for KnownCameraControls {
    fn from(mf_c: MediaFoundationControls) -> Self {
        match mf_c {
            MediaFoundationControls::Brightness => KnownCameraControls::Brightness,
            MediaFoundationControls::Contrast => KnownCameraControls::Contrast,
            MediaFoundationControls::Hue => KnownCameraControls::Hue,
            MediaFoundationControls::Saturation => KnownCameraControls::Saturation,
            MediaFoundationControls::Sharpness => KnownCameraControls::Sharpness,
            MediaFoundationControls::Gamma => KnownCameraControls::Gamma,
            MediaFoundationControls::ColorEnable => KnownCameraControls::ColorEnable,
            MediaFoundationControls::WhiteBalance => KnownCameraControls::WhiteBalance,
            MediaFoundationControls::BacklightComp => KnownCameraControls::BacklightComp,
            MediaFoundationControls::Gain => KnownCameraControls::Gain,
            MediaFoundationControls::Pan => KnownCameraControls::Pan,
            MediaFoundationControls::Tilt => KnownCameraControls::Tilt,
            MediaFoundationControls::Roll => KnownCameraControls::Roll,
            MediaFoundationControls::Zoom => KnownCameraControls::Zoom,
            MediaFoundationControls::Exposure => KnownCameraControls::Exposure,
            MediaFoundationControls::Iris => KnownCameraControls::Iris,
            MediaFoundationControls::Focus => KnownCameraControls::Focus,
        }
    }
}

#[cfg(feature = "input-msmf")]
impl From<MFControl> for KnownCameraControls {
    fn from(mf_cc: MFControl) -> Self {
        mf_cc.control().into()
    }
}

#[cfg(feature = "input-v4l")]
impl TryFrom<Description> for KnownCameraControls {
    type Error = NokhwaError;

    fn try_from(value: Description) -> Result<Self, Self::Error> {
        Ok(match value.id {
            9_963_776 => KnownCameraControls::Brightness,
            9_963_777 => KnownCameraControls::Contrast,
            9_963_779 => KnownCameraControls::Hue,
            9_963_778 => KnownCameraControls::Saturation,
            9_963_803 => KnownCameraControls::Sharpness,
            9_963_792 => KnownCameraControls::Gamma,
            9_963_802 => KnownCameraControls::WhiteBalance,
            9_963_804 => KnownCameraControls::BacklightComp,
            9_963_795 => KnownCameraControls::Gain,
            10_094_852 => KnownCameraControls::Pan,
            10_094_853 => KnownCameraControls::Tilt,
            10_094_862 => KnownCameraControls::Zoom,
            9_963_793 => KnownCameraControls::Exposure,
            10_094_866 => KnownCameraControls::Iris,
            10_094_859 => KnownCameraControls::Focus,
            _ => {
                return Err(NokhwaError::NotImplementedError(
                    "Control not implemented!".to_string(),
                ))
            }
        })
    }
}

/// This tells you weather a [`KnownCameraControls`] is automatically managed by the OS/Driver
/// or manually managed by you, the programmer.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum KnownCameraControlFlag {
    Automatic,
    Manual,
}

/// This struct tells you everything about a particular [`KnownCameraControls`]. <br>
/// However, you should never need to instantiate this struct, since its usually generated for you by `nokhwa`.
/// The only time you should be modifying this struct is when you need to set a value and pass it back to the camera.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct CameraControl {
    control: KnownCameraControls,
    min: i32,
    max: i32,
    value: i32,
    step: i32,
    default: i32,
    flag: KnownCameraControlFlag,
    active: bool,
}

impl CameraControl {
    /// Creates a new [`CameraControl`]
    /// # Errors
    /// If the `value` is below `min`, above `max`, or is not divisible by `step`, this will error
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        control: KnownCameraControls,
        minimum: i32,
        maximum: i32,
        value: i32,
        step: i32,
        default: i32,
        flag: KnownCameraControlFlag,
        active: bool,
    ) -> Result<Self, NokhwaError> {
        if value >= maximum {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too large".to_string(),
            });
        }
        if value <= minimum {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too low".to_string(),
            });
        }
        if value % step != 0 {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Not aligned with step".to_string(),
            });
        }

        Ok(CameraControl {
            control,
            min: minimum,
            max: maximum,
            value,
            step,
            default,
            flag,
            active,
        })
    }

    /// Gets the [`KnownCameraControls`] of this [`CameraControl`]
    #[must_use]
    pub fn control(&self) -> KnownCameraControls {
        self.control
    }

    /// Gets the minimum value of this [`CameraControl`]
    #[must_use]
    pub fn minimum_value(&self) -> i32 {
        self.min
    }

    /// Gets the maximum value of this [`CameraControl`]
    #[must_use]
    pub fn maximum_value(&self) -> i32 {
        self.max
    }

    /// Gets the current value of this [`CameraControl`]
    #[must_use]
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Sets the value of this [`CameraControl`]
    /// # Errors
    /// If the `value` is below `min`, above `max`, or is not divisible by `step`, this will error
    pub fn set_value(&mut self, value: i32) -> Result<(), NokhwaError> {
        if value >= self.maximum_value() {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too large".to_string(),
            });
        }
        if value <= self.minimum_value() {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too low".to_string(),
            });
        }
        if value % self.step() != 0 {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Not aligned with step".to_string(),
            });
        }

        self.value = value;
        Ok(())
    }

    /// Creates a new [`CameraControl`] but with `value`
    /// # Errors
    /// If the `value` is below `min`, above `max`, or is not divisible by `step`, this will error
    pub fn with_value(self, value: i32) -> Result<Self, NokhwaError> {
        if value >= self.maximum_value() {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too large".to_string(),
            });
        }
        if value <= self.minimum_value() {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Value too low".to_string(),
            });
        }
        if value % self.step() != 0 {
            return Err(NokhwaError::StructureError {
                structure: "CameraControl".to_string(),
                error: "Not aligned with step".to_string(),
            });
        }

        Ok(CameraControl {
            control: self.control(),
            min: self.minimum_value(),
            max: self.maximum_value(),
            value,
            step: self.step(),
            default: self.default(),
            flag: self.flag(),
            active: self.active(),
        })
    }

    /// Gets the step value of this [`CameraControl`]
    /// Note that `value` must be divisible by `step`
    #[must_use]
    pub fn step(&self) -> i32 {
        self.step
    }

    /// Gets the default value of this [`CameraControl`]
    #[must_use]
    pub fn default(&self) -> i32 {
        self.default
    }

    /// Gets the [`KnownCameraControlFlag`] of this [`CameraControl`],
    /// telling you weather this control is automatically set or manually set.
    #[must_use]
    pub fn flag(&self) -> KnownCameraControlFlag {
        self.flag
    }

    /// Gets `active` of this [`CameraControl`],
    /// telling you weather this control is currently active(in-use).
    #[must_use]
    pub fn active(&self) -> bool {
        self.active
    }

    /// Returns a list of i32s that are valid to be set.
    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    pub fn valid_values(&self) -> Vec<i32> {
        (self.minimum_value()..=self.maximum_value())
            .step_by(self.step() as usize)
            .into_iter()
            .collect()
    }
}

impl PartialOrd for CameraControl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CameraControl {
    fn cmp(&self, other: &Self) -> Ordering {
        self.control().cmp(&other.control())
    }
}

/// The list of known capture backends to the library. <br>
/// **Note: Only V4L2 and UVC (and by extension AUTO) is implemented so far.**
/// - `AUTO` is special - it tells the Camera struct to automatically choose a backend most suited for the current platform.
/// - `AVFoundation` - Uses `AVFoundation` on Mac **Not Implemted**
/// - `V4L2` - `Video4Linux2`, a linux specific backend.
/// - `UVC` - Universal Video Class (please check [libuvc](https://github.com/libuvc/libuvc)). Platform agnostic, although on linux it needs `sudo` permissions or similar to use.
/// - `MediaFoundation` - Microsoft Media Foundation, Windows only,
/// - `OpenCV` - Uses `OpenCV` to capture. Platform agnostic.
/// - `GStreamer` - Uses `GStreamer` RTP to capture. Platform agnostic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaptureAPIBackend {
    Auto,
    AVFoundation,
    Video4Linux,
    UniversalVideoClass,
    MediaFoundation,
    OpenCv,
    GStreamer,
}

impl Display for CaptureAPIBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let self_str = format!("{:?}", self);
        write!(f, "{}", self_str)
    }
}

/// Converts a MJPEG stream of [u8] into a Vec<u8> of RGB888. (R,G,B,R,G,B,...)
/// # Errors
/// If `mozjpeg` fails to read scanlines or setup the decompressor, this will error.
/// # Safety
/// This function uses `unsafe`. The caller must ensure that:
/// - The input data is of the right size, does not exceed bounds, and/or the final size matches with the initial size.
pub fn mjpeg_to_rgb888(data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
    let mut mozjpeg_decomp = match Decompress::new_mem(data) {
        Ok(decomp) => match decomp.rgb() {
            Ok(decompresser) => decompresser,
            Err(why) => {
                return Err(NokhwaError::ProcessFrameError {
                    src: FrameFormat::MJPEG,
                    destination: "RGB888".to_string(),
                    error: why.to_string(),
                })
            }
        },
        Err(why) => {
            return Err(NokhwaError::ProcessFrameError {
                src: FrameFormat::MJPEG,
                destination: "RGB888".to_string(),
                error: why.to_string(),
            })
        }
    };
    let decompressed = match mozjpeg_decomp.read_scanlines::<[u8; 3]>() {
        Some(pixels) => pixels,
        None => {
            return Err(NokhwaError::ProcessFrameError {
                src: FrameFormat::MJPEG,
                destination: "RGB888".to_string(),
                error: "Failed to get read readlines into RGB888 pixels!".to_string(),
            })
        }
    };

    Ok(unsafe { from_raw_parts(decompressed.as_ptr().cast(), decompressed.len() * 3) }.to_vec())
}

// For those maintaining this, I recommend you read: https://docs.microsoft.com/en-us/windows/win32/medfound/recommended-8-bit-yuv-formats-for-video-rendering#yuy2
// https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
// and this too: https://stackoverflow.com/questions/16107165/convert-from-yuv-420-to-imagebgr-byte
// The YUY2(YUYV) format is a 16 bit format. We read 4 bytes at a time to get 6 bytes of RGB888.
// First, the YUY2 is converted to YCbCr 4:4:4 (4:2:2 -> 4:4:4)
// then it is converted to 6 bytes (2 pixels) of RGB888
/// Converts a YUYV 4:2:2 datastream to a RGB888 Stream. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
/// # Errors
/// This may error when the data stream size is not divisible by 4, a i32 -> u8 conversion fails, or it fails to read from a certain index.
pub fn yuyv422_to_rgb888(data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
    let mut rgb_vec: Vec<u8> = vec![];
    if data.len() % 4 == 0 {
        for px_idx in (0..data.len()).step_by(4) {
            let y1 = match data.get(px_idx) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::ProcessFrameError { src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx, why.to_string()) });
                    }
                },
                None => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx
                        ),
                    });
                }
            };

            let u = match data.get(px_idx + 1) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::ProcessFrameError { src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx+1, why.to_string()) });
                    }
                },
                None => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx + 1
                        ),
                    });
                }
            };

            let y2 = match data.get(px_idx + 2) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::ProcessFrameError { src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx+2, why.to_string()) });
                    }
                },
                None => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx + 2
                        ),
                    });
                }
            };

            let v = match data.get(px_idx + 3) {
                Some(px) => match i32::try_from(*px) {
                    Ok(i) => i,
                    Err(why) => {
                        return Err(NokhwaError::ProcessFrameError { src: FrameFormat::YUYV, destination: "RGB888".to_string(), error: format!("Failed to convert byte at {} to a i32 because {}, This shouldn't happen!", px_idx+3, why.to_string()) });
                    }
                },
                None => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::YUYV,
                        destination: "RGB888".to_string(),
                        error: format!(
                            "Failed to get bytes at {}, this is probably a bug, please report!",
                            px_idx + 3
                        ),
                    });
                }
            };

            let pixel1 = yuyv444_to_rgb888(y1, u, v);
            let pixel2 = yuyv444_to_rgb888(y2, u, v);
            rgb_vec.append(&mut pixel1.to_vec());
            rgb_vec.append(&mut pixel2.to_vec());
        }
        Ok(rgb_vec)
    } else {
        Err(NokhwaError::ProcessFrameError {
            src: FrameFormat::YUYV,
            destination: "RGB888".to_string(),
            error: "Assertion failure, the YUV stream isn't 4:2:2! (wrong number of bytes)"
                .to_string(),
        })
    }
}

// equation from https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
/// Convert `YCbCr` 4:4:4 to a RGB888. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
#[allow(clippy::many_single_char_names)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[must_use]
pub fn yuyv444_to_rgb888(y: i32, u: i32, v: i32) -> [u8; 3] {
    let c298 = (y - 16) * 298;
    let d = u - 128;
    let e = v - 128;
    let r = ((c298 + 409 * e + 128) >> 8).clamp(0, 255) as u8;
    let g = ((c298 - 100 * d - 208 * e + 128) >> 8).clamp(0, 255) as u8;
    let b = ((c298 + 516 * d + 128) >> 8).clamp(0, 255) as u8;
    [r, g, b]
}

/// The `OpenCV` backend supports both native cameras and IP Cameras, so this is an enum to differentiate them
/// The `IPCamera`'s string follows the pattern
/// ```.ignore
/// <protocol>://<IP>:<port>/
/// ```
/// but please consult the manufacturer's specification for more details.
/// The index is a standard webcam index.
#[derive(Clone, Debug, PartialEq)]
pub enum CameraIndexType {
    Index(u32),
    IPCamera(String),
}

impl Display for CameraIndexType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CameraIndexType::Index(idx) => {
                write!(f, "{}", idx)
            }
            CameraIndexType::IPCamera(ip) => {
                write!(f, "{}", ip)
            }
        }
    }
}
