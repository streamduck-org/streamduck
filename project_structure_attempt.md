### Core
- plugin_sources (PluginSource)
- plugins (Plugin)
- devices (Device)

### PluginSource
- load_plugins() -> [Plugin]

### Plugin
- init()
- drivers (Driver)
- actions (Action)
- overlays (Overlay)

### Driver
- list_devices() -> DeviceInformation
- connect_device(DeviceInformation) -> Device

### Device
- disconnect()
- inputs() -> [Input]
- outputs() -> [Output]

### Input
- capabilities (InputCapability)
- icon_hint (InputIconHint)
- events (InputEvent)
- position (i32, i32)
- size (u32, u32)

### InputCapability (bitmask)
- Button
- Knob
- Slider
- Latched
- Pressured
- Infinite
- XY
- Relative
- Continuous

#### Various inputs explained as Capability bitmask
```
Toggle = Button | Latched
Analog Button = Button | Pressured
Encoder = Knob | Infinite
TouchScreen = XY | Button
Touchpad = XY | Relative | Button
Joystick = XY | Continuous | Relative | Button
Trackball = XY | Continuous | Relative
```

### InputIconHint
- Guess (defaults to using Capability to guess)
- Button
- Toggle
- AnalogButton
- Slider
- Knob
- Encoder
- TouchScreen
- Joystick
- Trackball
- Touchpad
- Sensor

### InputEvent
- name
- description
- signature ([ValueType])

### Output
- capabilities (OutputCapability)
- icon_hint