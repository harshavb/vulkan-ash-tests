use ash::Instance;
use ash::{vk, Entry};
use std::error::Error;
use std::ffi::CString;
use winit::window::Window;

pub struct VulkanBase {
    _entry: Entry,
    instance: Instance,
}

impl VulkanBase {
    pub fn new(window: &Window) -> Result<VulkanBase, Box<dyn Error>> {
        let (_entry, instance) = VulkanBase::create_instance(window)?;
        VulkanBase::pick_physical_device();
        Ok(VulkanBase { _entry, instance })
    }

    // Creates an ash Instance, which is a light wrapper around a vk::Instance
    fn create_instance(window: &Window) -> Result<(Entry, Instance), Box<dyn Error>> {
        // Specifies extensions
        let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
        let extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        // Loads names into CStrings
        let application_name = CString::new("Hello Triangle")?;
        let engine_name = CString::new("Hello Triangle Engine")?;

        // Creates application info
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&application_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::make_api_version(0, 1, 0, 0));

        // Creates instance info
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names_raw);

        // Creats weird wrapper type for accessing cpp vulkan dynamic library, and creates an ash instance inside
        let entry = unsafe { Entry::new()? };
        let instance = unsafe { entry.create_instance(&create_info, None)? };
        return Ok((entry, instance));
    }

    fn pick_physical_device() {
        
    }
}

impl Drop for VulkanBase {
    fn drop(&mut self) {
        println!("Cleaning up VulkanBase!");
        unsafe {
            self.instance.destroy_instance(None);
        }
        println!("Cleaned up VulkanBase!");
    }
}