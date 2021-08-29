pub use crate::graphics::graphics_errors::GraphicsError;
use ash::{
    extensions::khr::{Surface, Swapchain},
    util, vk, Device, Entry, Instance,
};
use std::{
    ffi::{CStr, CString},
    io::Cursor,
    vec::Vec,
};
use winit::window::Window;

const BAD_ERROR: &str = "Something went incredibly wrong!";

pub struct VulkanBase {
    _entry: Entry,
    instance: Instance,
    surface_khr: vk::SurfaceKHR,
    surface: Surface,
    device: Device,
    swapchain_khr: vk::SwapchainKHR,
    swapchain: Swapchain,
    swapchain_image_views: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    shader_modules: Vec<vk::ShaderModule>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

pub struct WindowDimensions {
    width: u32,
    height: u32,
}

impl WindowDimensions {
    pub fn new(width: u32, height: u32) -> WindowDimensions {
        WindowDimensions { width, height }
    }
}

// Assumes graphics and presentation queue families are the same
struct QueueFamilyIndices {
    queue_family_index: u32,
}

struct SwapchainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    presentation_modes: Vec<vk::PresentModeKHR>,
}

struct SwapchainDetails {
    format: vk::SurfaceFormatKHR,
    _presentation_mode: vk::PresentModeKHR,
    extent: vk::Extent2D,
}

impl VulkanBase {
    pub fn new(window: &Window, window_dimensions: WindowDimensions) -> VulkanBase {
        // Creates Entry and Instance
        let (_entry, instance) = VulkanBase::create_instance(&window);

        // Creates vk::SurfaceKHR and Surface
        let (surface_khr, surface) = VulkanBase::create_surface(&_entry, &instance, &window);

        // Stores necessary device extensions
        let device_extension_names_raw = [Swapchain::name().as_ptr()];

        // Creates PhysicalDevice and stores queue family indices
        let (physical_device, queue_family_indices, swapchain_support_details) =
            VulkanBase::pick_physical_device(
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

        // Creates vk::SwapchainKHR, Swapchain, and SwapchainDetails
        let (swapchain_khr, swapchain, swapchain_details) = VulkanBase::create_swapchain(
            &instance,
            &device,
            &window_dimensions,
            &surface_khr,
            &swapchain_support_details,
        );

        // Retreives available swapchain images
        let swapchain_images = unsafe {
            swapchain
                .get_swapchain_images(swapchain_khr)
                .expect(BAD_ERROR)
        };

        // Creates and stores an image view for each swapchain image
        let swapchain_image_views =
            VulkanBase::create_image_views(&device, &swapchain_images, &swapchain_details.format);

        // Creates a queue handle for the queue family (assumes both the graphics and presentation queue families are the same)
        let _queue = unsafe { device.get_device_queue(queue_family_indices.queue_family_index, 0) };

        let render_pass = VulkanBase::create_render_pass(&device, &swapchain_details.format);

        // Creates shader modules, pipeline layout, and pipeline
        let (shader_modules, pipeline_layout, pipeline) =
            VulkanBase::create_graphics_pipeline(&device, &render_pass, &swapchain_details.extent);

        VulkanBase {
            _entry,
            instance,
            surface_khr,
            surface,
            device,
            swapchain_khr,
            swapchain,
            swapchain_image_views,
            render_pass,
            shader_modules,
            pipeline_layout,
            pipeline,
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
        let entry = unsafe { Entry::new().expect(BAD_ERROR) };

        // Creates ash instance
        let instance = unsafe { entry.create_instance(&create_info, None).expect(BAD_ERROR) };

        (entry, instance)
    }

    // Creates a window surface
    fn create_surface(
        entry: &Entry,
        instance: &Instance,
        window: &Window,
    ) -> (vk::SurfaceKHR, Surface) {
        // Creates a Vulkan KHR object and then a light ash wrapper (both are required)
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
    ) -> (
        vk::PhysicalDevice,
        QueueFamilyIndices,
        SwapchainSupportDetails,
    ) {
        let physical_devices = unsafe { instance.enumerate_physical_devices().expect(BAD_ERROR) };

        for device in physical_devices {
            if let Some((queue_family_indices, swapchain_support_details)) =
                VulkanBase::is_device_suitable(instance, &device, extensions, surface_khr, surface)
            {
                return (device, queue_family_indices, swapchain_support_details);
            }
        }

        panic!("No valid GPU!");
    }

    // Checks whether a given physical device is valid, and if it is returns the queue family indices of that device
    fn is_device_suitable(
        instance: &Instance,
        device: &vk::PhysicalDevice,
        required_extensions: &[*const i8],
        surface_khr: &vk::SurfaceKHR,
        surface: &Surface,
    ) -> Option<(QueueFamilyIndices, SwapchainSupportDetails)> {
        if !VulkanBase::check_device_extension_support(instance, device, required_extensions) {
            return None;
        }

        let swapchain_support_details =
            VulkanBase::query_swapchain_support_details(device, surface_khr, surface);

        if swapchain_support_details.formats.is_empty()
            | swapchain_support_details.presentation_modes.is_empty()
        {
            return None;
        }

        let queue_family_indices =
            VulkanBase::find_queue_families(instance, device, surface_khr, surface)?;

        Some((queue_family_indices, swapchain_support_details))
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
                .expect(BAD_ERROR)
        };

        // Unsure if this is faster than using a hashset - device_extensions has length 122 on my system
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

    // Gets a given physical device's surface capabilities, formats, and presentation modes
    fn query_swapchain_support_details(
        device: &vk::PhysicalDevice,
        surface_khr: &vk::SurfaceKHR,
        surface: &Surface,
    ) -> SwapchainSupportDetails {
        let capabilities = unsafe {
            surface
                .get_physical_device_surface_capabilities(*device, *surface_khr)
                .expect(BAD_ERROR)
        };

        let formats = unsafe {
            surface
                .get_physical_device_surface_formats(*device, *surface_khr)
                .expect(BAD_ERROR)
        };

        let presentation_modes = unsafe {
            surface
                .get_physical_device_surface_present_modes(*device, *surface_khr)
                .expect(BAD_ERROR)
        };

        SwapchainSupportDetails {
            capabilities,
            formats,
            presentation_modes,
        }
    }

    // Creates the logical device based on necessary queue families
    fn create_logical_device(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        extensions: &[*const i8],
        indices: &QueueFamilyIndices,
    ) -> Device {
        let queue_priorities = [1.0];

        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.queue_family_index)
            .queue_priorities(&queue_priorities);

        // Wraps reference to queue_info in a slice in order to preserve lifetime information
        // Bad alternative is to build() the queue_info
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(extensions);

        let device = unsafe {
            instance
                .create_device(*physical_device, &device_create_info, None)
                .expect(BAD_ERROR)
        };

        device
    }

    // Creates the swap chain after determining swap chain settings
    fn create_swapchain(
        instance: &Instance,
        device: &Device,
        window: &WindowDimensions,
        surface: &vk::SurfaceKHR,
        swapchain_support_details: &SwapchainSupportDetails,
    ) -> (vk::SwapchainKHR, Swapchain, SwapchainDetails) {
        let format = VulkanBase::choose_swap_surface_format(&swapchain_support_details.formats);

        let presentation_mode = VulkanBase::choose_swap_surface_presentation_mode(
            &swapchain_support_details.presentation_modes,
        );

        let extent =
            VulkanBase::choose_swap_extent(window, &swapchain_support_details.capabilities);

        let image_count = swapchain_support_details.capabilities.min_image_count;

        // Implements double buffering
        /*if swapchain_support_details.capabilities.max_image_count
            != swapchain_support_details.capabilities.min_image_count
        {
            image_count += 1;
        }*/

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(swapchain_support_details.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(presentation_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = Swapchain::new(instance, device);
        let swapchain_khr = unsafe {
            swapchain
                .create_swapchain(&swapchain_create_info, None)
                .expect(BAD_ERROR)
        };

        let swapchain_details = SwapchainDetails {
            format,
            _presentation_mode: presentation_mode,
            extent: extent,
        };

        (swapchain_khr, swapchain, swapchain_details)
    }

    // Determines surface format
    fn choose_swap_surface_format(formats: &Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
        for format in formats {
            if format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *format;
            }
        }

        *formats.get(0).expect("No available surface formats!")
    }

    // Chooses presentation mode - immediate is preferred for least latency (as opposed to VSync aka FIFO)
    fn choose_swap_surface_presentation_mode(
        presentation_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for presentation_mode in presentation_modes {
            if *presentation_mode == vk::PresentModeKHR::IMMEDIATE {
                return *presentation_mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    // Creates an extent with the correct size
    fn choose_swap_extent(
        window: &WindowDimensions,
        capabilities: &vk::SurfaceCapabilitiesKHR,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != std::u32::MAX {
            return capabilities.current_extent;
        }

        vk::Extent2D {
            width: window.width,
            height: window.height,
        }
    }

    // Creates an image view for each image in the swapchain
    fn create_image_views(
        device: &Device,
        images: &Vec<vk::Image>,
        format: &vk::SurfaceFormatKHR,
    ) -> Vec<vk::ImageView> {
        images
            .iter()
            .map(|image| {
                let image_view_create_info = vk::ImageViewCreateInfo::builder()
                    .image(*image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe {
                    device
                        .create_image_view(&image_view_create_info, None)
                        .expect(BAD_ERROR)
                }
            })
            .collect()
    }

    fn create_render_pass(device: &Device, format: &vk::SurfaceFormatKHR) -> vk::RenderPass {
        let color_attachments = [vk::AttachmentDescription::builder()
            .format(format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build()];

        let color_attachment_references = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];

        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_references)
            .build()];

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&color_attachments)
            .subpasses(&subpasses);

        unsafe {
            device
                .create_render_pass(&render_pass_info, None)
                .expect(BAD_ERROR)
        }
    }

    // Creates shader modules, graphics pipeline layout, and graphics pipeline
    fn create_graphics_pipeline(
        device: &Device,
        render_pass: &vk::RenderPass,
        extent: &vk::Extent2D,
    ) -> (Vec<vk::ShaderModule>, vk::PipelineLayout, vk::Pipeline) {
        let (vertex_code, fragment_code) = VulkanBase::read_shaders();

        // Shader modules
        let vertex_shader_module = VulkanBase::create_shader_module(device, &vertex_code);
        let fragment_shader_module = VulkanBase::create_shader_module(device, &fragment_code);

        let shader_entry_name = CString::new("main").unwrap();

        let shader_stage_infos = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(vertex_shader_module)
                .name(shader_entry_name.as_c_str())
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(fragment_shader_module)
                .name(shader_entry_name.as_c_str())
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[])
            .vertex_attribute_descriptions(&[]);

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        // In reality viewport and scissors should be set during render pass dynamically, rather than before,
        // in order to prevent having to recreate the pipeline everytime the window is resized
        let viewports = [vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build()];

        let scissors = [vk::Rect2D::builder()
            .offset(*vk::Offset2D::builder().x(0).y(0))
            .extent(*extent)
            .build()];

        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let alpha_blending_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::all())
            .build()];

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&alpha_blending_attachments);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect(BAD_ERROR)
        };

        let graphics_pipeline_info = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_infos)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_info)
            .layout(pipeline_layout)
            .render_pass(*render_pass)
            .build()];

        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &graphics_pipeline_info, None)
                .expect(BAD_ERROR)
        };

        (
            vec![vertex_shader_module, fragment_shader_module],
            pipeline_layout,
            graphics_pipelines[0],
        )
    }

    // Reads shader code from shader files (should be changed to take a shader file path as an argument in production code... I think)
    fn read_shaders() -> (Vec<u32>, Vec<u32>) {
        // Reads precompiled shaders
        // Macro include_bytes! must know path names at compile time! If shaders are unknown, or for automation, a different solution is required.
        let mut vertex_shader_file = Cursor::new(&include_bytes!("shaders\\vertex.spv"));
        let mut fragment_shader_file = Cursor::new(&include_bytes!("shaders\\fragment.spv"));

        let vertex_code =
            util::read_spv(&mut vertex_shader_file).expect("Failed to read vertex shader file");
        let fragment_code =
            util::read_spv(&mut fragment_shader_file).expect("Failed to read fragment shader file");

        (vertex_code, fragment_code)
    }

    // Creates a shader module from shader code stored in a u32 vector
    fn create_shader_module(device: &Device, code: &Vec<u32>) -> vk::ShaderModule {
        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(code);

        unsafe {
            device
                .create_shader_module(&shader_module_create_info, None)
                .expect(BAD_ERROR)
        }
    }
}

impl Drop for VulkanBase {
    fn drop(&mut self) {
        println!("Cleaning up VulkanBase!");
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for shader_module in self.shader_modules.iter() {
                self.device.destroy_shader_module(*shader_module, None);
            }
            for image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(*image_view, None);
            }
            self.swapchain.destroy_swapchain(self.swapchain_khr, None);
            self.device.destroy_device(None);
            self.surface.destroy_surface(self.surface_khr, None);
            self.instance.destroy_instance(None);
        }
        println!("Cleaned up VulkanBase!");
    }
}
