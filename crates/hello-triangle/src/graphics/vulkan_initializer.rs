use ash::vk;

pub struct VulkanType {
    instance: vk::Instance,
}

impl VulkanType {
    pub fn new() -> Result<VulkanType, Box<dyn Error>> {
        vk_instance = createInstance()?;
    }
}

fn createInstance(window: &Window) -> Result<vk::Instance, Box<dyn Error>> {
    let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
    let mut extension_names_raw = surface_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();
    let appInfo = vk::ApplicationInfo::builder()
        .application_name(CString::new("Hello Triangle").unwrap()?)
        .application_version(vk::make_version(0, 1, 0))
        .engine_name(CString::new("Hello Triangle Engine").unwrap()?)
        .engine_version(vk::make_version(0, 1, 0))
        .api_version(vk::make_api_version(0, 1, 0, 0));
    let createInfo = vk::InstanceCreateInfo::builder()
        .application_info(&appInfo)
        .enabled_extension_names(&extension_names_raw)
        .enabled_layer_count(0);
}
