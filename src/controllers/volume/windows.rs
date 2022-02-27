use self::winguids::*;
use core::mem::MaybeUninit;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Media::Audio::Endpoints::*;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;

mod winguids {
    #![allow(non_upper_case_globals, dead_code)]

    use windows::core::GUID;

    pub const IID_IAudioEndpointVolume: GUID = GUID {
        data1: 0x5cdf2c82,
        data2: 0x841e,
        data3: 0x4546,
        data4: [0x97, 0x22, 0x0c, 0xf7, 0x40, 0x78, 0x22, 0x9a],
    };

    pub const IID_IAudioSessionManager2: GUID = GUID {
        data1: 0x77aa99a0,
        data2: 0x1bd6,
        data3: 0x484f,
        data4: [0x8b, 0xc7, 0x2c, 0x65, 0x4c, 0x9a, 0x9b, 0x6f],
    };
}

pub struct VolumeControllerImpl {
    volume: IAudioEndpointVolume,
}

impl VolumeControllerImpl {
    // this was my first time doing COM programming, and lemme tell ya, it sure is
    // *something* alright...
    pub fn new_system_default() -> Result<VolumeControllerImpl> {
        unsafe {
            CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED)?;

            let device_enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

            let device = device_enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;

            let volume = {
                let mut obj: MaybeUninit<IAudioEndpointVolume> = MaybeUninit::uninit();
                device.Activate(
                    &IID_IAudioEndpointVolume,
                    CLSCTX_ALL,
                    core::ptr::null(),
                    obj.as_mut_ptr() as _,
                )?;
                obj.assume_init()
            };

            // let audio_session_manager = {
            //     let mut obj: MaybeUninit<IAudioSessionManager2> = MaybeUninit::uninit();
            //     device.Activate(
            //         &IID_IAudioSessionManager2,
            //         CLSCTX_ALL,
            //         core::ptr::null(),
            //         obj.as_mut_ptr() as _,
            //     )?;
            //     obj.assume_init()
            // };

            // let session_enumerator = audio_session_manager.GetSessionEnumerator()?;

            // for session in
            //     (0..session_enumerator.GetCount()?).map(|id|
            // session_enumerator.GetSession(id)) {
            //     let session = session?;

            //     let name = read_to_string(session.GetDisplayName()?);
            //     dbg!(name);
            // }

            Ok(VolumeControllerImpl { volume })
        }
    }

    pub fn get_master_volume(&self) -> Result<f32> {
        unsafe { self.volume.GetMasterVolumeLevelScalar() }
    }

    pub fn set_master_volume(&self, vol: f32) -> Result<()> {
        unsafe {
            self.volume
                .SetMasterVolumeLevelScalar(vol, core::ptr::null())
        }
    }
}

#[allow(dead_code)]
unsafe fn read_to_string(ptr: PWSTR) -> String {
    let mut len = 0usize;
    let mut cursor = ptr;
    loop {
        let val = cursor.0.read();
        if val == 0 {
            break;
        }
        len += 1;
        cursor = PWSTR(cursor.0.add(1));
    }

    let slice = std::slice::from_raw_parts(ptr.0, len);
    String::from_utf16(slice).unwrap()
}
