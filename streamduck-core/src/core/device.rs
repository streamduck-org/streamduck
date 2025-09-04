use std::collections::HashSet;
use std::rc::Rc;
use std::time::Instant;
use log::{error, info};
use crate::api::device::{BoxDevice, DeviceError};
use crate::api::driver::{DeviceConnectionError, NamespacedDeviceIdentifier};
use crate::config::CONFIG;
use crate::core::Core;
use crate::core::events::CoreEvent;

pub struct ManagedDevice<'core> {
    core: &'core Core<'core>,
    device: BoxDevice,
    id: NamespacedDeviceIdentifier
}

impl<'core> ManagedDevice<'core> {
    pub fn new(core: &'core Core<'core>, device: BoxDevice, id: NamespacedDeviceIdentifier) -> Self {
        Self {
            core,
            device,
            id,
        }
    }

    pub fn tick(&self) -> Result<(), DeviceError> {
        let events = self.device.tick()?;

        // do something with the events I guess
        for event in events {
            println!("{:?}", event);
        }

        Ok(())
    }
}

impl<'core> Core<'core> {
    pub fn discover_devices(&'core self) {
        info!("Running device discovery...");
        let mut new_discovered = HashSet::new();

        for (id, driver) in self.drivers.borrow().iter() {
            let devices = match driver.list_devices() {
                Ok(v) => v,
                Err(e) => {
                    error!("Error while trying to list devices on '{}':\n{}", id, e);
                    continue
                }
            }.into_iter()
                .map(|device| device.namespaced(id.clone()));

            new_discovered.extend(devices);
        }

        let mut old_discovered = self.discovered_devices.borrow_mut();

        // Discovering new devices
        for device in new_discovered.iter() {
            if !old_discovered.contains(device) {
                info!("New device discovered: {}", device);
                self.send_event(CoreEvent::DiscoveredDevice(device.clone()));

                if let Err(err) = self.connect_device(device) {
                    error!("Failed to connect: {}", err);
                }
            }
        }

        // Pointing out lost devices
        for device in old_discovered.iter() {
            if !new_discovered.contains(device) {
                info!("Device lost: {}", device);
                self.send_event(CoreEvent::DiscoveredDeviceLost(device.clone()))
            }
        }

        self.last_discovery.replace(Some(Instant::now()));
        *old_discovered = new_discovered;
    }

    pub fn check_device_discovery(&'core self) {
        let Some(last_time) = self.last_discovery.borrow().clone() else {
            self.discover_devices();
            return;
        };

        if last_time.elapsed() > CONFIG.load().device_discovery_interval {
            self.discover_devices();
        }
    }

    pub fn connect_device(&'core self, device_id: &NamespacedDeviceIdentifier) -> Result<Rc<ManagedDevice<'core>>, DeviceConnectionError> {
        let drivers = self.drivers.borrow();

        let Some(driver) = drivers.get(&device_id.driver) else {
            return Err(DeviceConnectionError::DriverNotFound)
        };

        info!("Attempting to connect to '{}'", device_id);
        let device = driver.connect_device(&device_id.id)?;

        info!("Successfully connected to '{}'", device_id);
        let managed = Rc::new(ManagedDevice::new(self, device, device_id.clone()));

        self.devices.borrow_mut().insert(device_id.clone(), managed.clone());
        Ok(managed)
    }

    pub fn tick_connected_devices(&'core self) {
        let devices = self.devices.borrow().values()
            .cloned().collect::<Vec<_>>();

        for device in devices {
            if let Err(e) = device.tick() {
                error!("Error while ticking device '{}':\n{}", device.id, e);
                self.devices.borrow_mut().remove(&device.id);
            }
        }
    }
}