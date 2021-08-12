pub use crate::graphics::graphics_errors::GraphicsError;
use ash::{
    extensions::khr::{Surface, Swapchain},
    vk, Device, Entry, Instance,
};
use std::ffi::{CStr, CString};
use winit::window::Window;

pub struct VulkanBase {
    _entry: Entry,
    instance: Instance,
    surface_khr: vk::SurfaceKHR,
    surface: Surface,
    device: Device,
}

// Assumes graphics and presentation queue families are the same
struct QueueFamilyIndices {
    queue_family_index: u32,
}

impl VulkanBase {
    pub fn new(window: &Window) -> VulkanBase {
        // Creates Entry and Instance
        let (_entry, instance) = VulkanBase::create_instance(window);

        // Creates vk::SurfaceKHR and Surface
        let (surface_khr, surface) = VulkanBase::create_surface(&_entry, &instance, &window);

        let device_extension_names_raw = [Swapchain::name().as_ptr()];

        // Creates PhysicalDevice and stores queue family indices
        let (physical_device, queue_family_indices) = VulkanBase::pick_physical_device(
            &instance,
            &device_extension_names_raw,
            &surface_khr,
            &surface,
        );

        // Creates Device
        let device = VulkanBase::create_logical_device(
            &instance,
            &physical_device,
            &device_extension_names_raw,
            &queue_family_indices,
        );

        // Creates a queue handle for the queue family (assumes both the graphics and presentation queue families are the same)
        let _queue = unsafe { device.get_device_queue(queue_family_indices.queue_family_index, 0) };

        VulkanBase {
            _entry,
            instance,
            surface_khr,
            surface,
            device,
        }
    }

    // Creates an ash Instance, which is a light wrapper around a vk::Instance
    fn create_instance(window: &Window) -> (Entry, Instance) {
        // Specifies extensions
        let surface_extensions =
            ash_window::enumerate_required_extensions(window).expect("Unsupported platform!");
        let extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        // Loads names into CStrings
        let application_name = CString::new("Hello Triangle").unwrap();
        let engine_name = CString::new("Hello Triangle Engine").unwrap();

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

        // Creats weird wrapper type for accessing cpp vulkan dynamic library
        let entry = unsafe { Entry::new().expect("Something went incredibly wrong!") };

        // Creates ash instance
        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Something went incredibly wrong!")
        };

        (entry, instance)
    }

    // Creates a window surface
    fn create_surface(
        entry: &Entry,
        instance: &Instance,
        window: &Window,
    ) -> (vk::SurfaceKHR, Surface) {
        let surface_khr = unsafe {
            ash_window::create_surface(entry, instance, window, None)
                .expect("Unsupported platform!")
        };
        let surface = Surface::new(entry, instance);
        (surface_khr, surface)
    }

    // Picks the first valid physical device
    fn pick_physical_device(
        instance: &Instance,
        extensions: &[*const i8],
        surface_khr: &vk::SurfaceKHR,
        surface: &Surface,
    ) -> (vk::PhysicalDevice, QueueFamilyIndices) {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Something went incredibly wrong!")
        };
        for device in physical_devices {
            if let Some(value) =
                VulkanBase::is_device_suitable(instance, &device, extensions, surface_khr, surface)
            {
                return (device, value);
            }
        }
        panic!("No valid GPU!");
    }

    // Checks whether a given physical device is valid, and returns the queue family indices of that device
    fn is_device_suitable(
        instance: &Instance,
        device: &vk::PhysicalDevice,
        required_extensions: &[*const i8],
        surface_khr: &vk::SurfaceKHR,
        surface: &Surface,
    ) -> Option<QueueFamilyIndices> {
        if !VulkanBase::check_device_extension_support(instance, device, required_extensions) {
            return None;
        }

        let queue_family_indices =
            VulkanBase::find_queue_families(instance, device, surface_khr, surface);

        queue_family_indices
    }

    // Checcks if a given physical device supports given device extensions
    fn check_device_extension_support(
        instance: &Instance,
        device: &vk::PhysicalDevice,
        required_extensions: &[*const i8],
    ) -> bool {
        let device_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(*device)
                .expect("Something went incredibly wrong!")
        };

        required_extensions.iter().all(|required_extension| {
            let required_extension_name = unsafe { CStr::from_ptr(*required_extension) };

            device_extensions.iter().any(|device_extension| {
                let device_extension_name =
                    unsafe { CStr::from_ptr(device_extension.extension_name.as_ptr()) };

                required_extension_name == device_extension_name
            })
        })
    }

    // Finds the queue families of a given physical device
    fn find_queue_families(
        instance: &Instance,
        device: &vk::PhysicalDevice,
        surface_khr: &vk::SurfaceKHR,
        surface: &Surface,
    ) -> Option<QueueFamilyIndices> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*device) };

        for (index, queue_family) in queue_families.iter().enumerate() {
            // Checks for graphics queue family
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                // Checks for presentation queue family
                && unsafe {
                    surface
                        .get_physical_device_surface_support(*device, index as u32, *surface_khr)
                        .unwrap()
                }
            {
                // Assumes both queue families are the same
                return Some(QueueFamilyIndices {
                    queue_family_index: index as u32,
                });
            }
        }
        None
    }

    // Creates the logical device based on necessary queue families
    fn create_logical_device(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        extensions: &[*const i8],
        indices: &QueueFamilyIndices,
    ) -> Device {
        let queue_priorities = [1.0];

        let queue_info = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.queue_family_index)
            .queue_priorities(&queue_priorities)
            .build()];

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(extensions);

        let device = unsafe {
            instance
                .create_device(*physical_device, &device_create_info, None)
                .expect("Something went incredibly wrong!")
        };

        device
    }
}

impl Drop for VulkanBase {
    fn drop(&mut self) {
        println!("Cleaning up VulkanBase!");
        unsafe {
            self.device.destroy_device(None);
            self.surface.destroy_surface(self.surface_khr, None);
            self.instance.destroy_instance(None);
        }
        println!("Cleaned up VulkanBase!");
    }
}
