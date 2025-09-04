---@meta Streamduck Lua API

---@alias DeviceIdentifier [string, string, any] Unique identifier for a device, first value is serial number, second is display name/product name, third is any serializable data for additional identification (ie. product id)

---@class LuaDriver Lua table that's expected to behave like a driver for Streamduck
local luaDriver = {}

---Lists devices the driver could connect to
---@return DeviceIdentifier[]
function luaDriver:listDevices() end

---@class PluginRegistry Registry of all the possible things a plugin could define
local pluginRegistry = {}

---Declares a driver to Streamduck with table serving as the driver interface
---@param name string
---@param driver LuaDriver
function pluginRegistry:registerDriver(name, driver) end

---@alias ValueType # Description of a possible value
---| '"Any"' # Any value
---| '"Boolean"' # Boolean value
---| '"Integer"' # Integer that can be negative
---| '"UnsignedInteger"' # Integer that can only be positive
---| '"Double"' # Any real number
---| '"String"' # String value
---| { Nullable: ValueType } # Value that can be nil
---| { Union: ValueType[] } # Value that can one of the specified types
---| { Tuple: ValueType[] } # An tuple that contains a number of different types
---| { Array: ValueType } # Array of a specified type
---| { Map: ValueType } # Map of string keys and specified value type
---| { Object: { [string]: ValueType } } # A structure/object

---@class InputEvent Description of a possible event that a device's input could send
---@field name string Name of the event
---@field description string? Description of the event
---@field signature ValueType Value type of the event
local inputEvent = {}

---@alias DeviceInputIcon # Icon of the input
---| '"Button"'
---| '"Toggle"'
---| '"AnalogButton"'
---| '"Slider"'
---| '"Knob"'
---| '"Encoder"'
---| '"TouchScreen"'
---| '"Joystick"'
---| '"Trackball"'
---| '"Touchpad"'
---| '"Sensor"'

---@class DeviceInput Explanation of a device input
---@field position_size [[number, number], [number, number]] Position and size of the input, position expects signed integers, size expects unsigned integers
---@field icon DeviceInputIcon
---@field events InputEvent[] Possible events that this input could create
local deviceInput = {}

---@alias DeviceOutputCapability # Explains what this output can do
---| {Display: [number, number]} # Display with specified size, expects unsigned integers
---| '"Light"' # Light that can be turned on and off
---| '"RGBLight"' # RGB light that accepts an RGB color

---@class DeviceOutput Explanation of a device output
---@field position_size [[number, number], [number, number]]? Position and size of the input, position expects signed integers, size expects unsigned integers. Can be used in case position of the output is different from input
---@field attached_to_input number? Index of an input if this output is integrated into the input, allows actions from input to draw to the output
---@field capabilities DeviceOutputCapability[] What this output can do
local deviceOutput = {}

---@class DeviceLayout Physical layout of the device
---@field name string Name of the layout in case device can have different configurations
---@field inputs DeviceInput[] Inputs of the device
---@field outputs DeviceOutput[] Outputs of the device
local deviceLayout = {}

---Connects to a device that was previously listed with listDevices method
---@param id DeviceIdentifier
---@return LuaDevice
function luaDriver:connectDevice(id) end

---@class LuaDevice Lua table that's expected to behave like a device for Streamduck
local luaDevice = {}

---Should destroy whatever data associated with the device
function luaDevice:disconnect() end

---@class DeviceEventCollector Collects whatever device events that might have occured during the tick
local deviceEventCollector = {}

---Adds input event into the collector
---@param index number Index of the input that made the event
---@param name string Name of the event
---@param value any Data of the event
function deviceEventCollector:input(index, name, value) end

---Adds layout change event into the collector, making the software query for device layout again
function deviceEventCollector:layoutChanged() end

---Ticks the device and collects any events it might have
---@param collector DeviceEventCollector
function luaDevice:tick(collector) end

---Returns the physical layout of the device
---@return DeviceLayout
function luaDevice:layout() end

---@class Common Common definitions for Streamduck API
---@field buttonEvent InputEvent A simple button up or down event
---@field toggleEvent InputEvent Toggle event with state of the toggle
---@field analogButtonEvent InputEvent Button with pressure sensitivity, should pass values between 0 and 1
---@field sliderEvent InputEvent Slider event, should pass values between 0 and 1
---@field knobEvent InputEvent Finite knob event, should pass values between 0 and 1
---@field encoderEvent InputEvent Infinite knob/encoder event, should pass positive integer when turned clockwise, negative when counter clockwise
---@field touchScreenPressEvent InputEvent Touch screen press event, should pass two unsigned integers and potential press state
---@field touchScreenLongPressEvent InputEvent Touch screen long press event, should pass two unsigned integers
---@field touchScreenDragEvent InputEvent Touch screen drag event, drag should start from touchScreenPress and end with that having false state, should pass two unsigned integers
---@field touchScreenSwipeEvent InputEvent Touch screen swipe event, for touch screens that can't tell current state of dragging, just indicates finger moving from one spot to another
---@field joystickEvent InputEvent Joystick move event, should be passed whenever joystick changes position, expecting floating values from -1 to 1
---@field touchpadEvent InputEvent Touchpad relative move event, expecting 2 integers indicating distance in pixels
common = {}

---Lua interface of HIDAPI
hid = {}

---Refreshes the HID device list
function hid:refresh() end

---Checks how much seconds passed since refresh was last called
---@return number
function hid:timeSinceRefresh() end

---Lists HID devices, optionally filters by vendor and product IDs if specified
---@param vid number? Vendor ID
---@param pid number? Product ID
---@return HIDDeviceInfo[]
function hid:list(vid, pid) end

---Opens connection to the device
---@param vid number Vendor ID
---@param pid number Product ID
---@param serial string? Serial number
---@return HIDDevice
function hid:open(vid, pid, serial) end

---@class HIDDevice Userdata containing the HID Device
local hidDevice = {}

---Gets feature report of specified size from the device
---@param amount number Size of the buffer to receive
---@return ByteArray bytes Byte array containing the feature report
function hidDevice:getFeatureReport(amount) end

---Sends feature report to the device
---@param bytes ByteArray | number[] Byte array to send
function hidDevice:sendFeatureReport(bytes) end

---Reads data from the device
---@param amount number Amount of bytes to attempt to read from the device
---@return number written Amount of bytes that were written
---@return ByteArray bytes Byte array containing the data
function hidDevice:read(amount) end

---Writes data to the device
---@param bytes ByteArray | number[] Data to write
function hidDevice:write(bytes) end

---Get indexed string from the device
---@param index number Index of the string
---@return string?
function hidDevice:getIndexedString(index) end

---Get device info
---@return HIDDeviceInfo
function hidDevice:info() end

---Consumes the device interface and destroys it
function hidDevice:destroy() end

---@class HIDDeviceInfo HID Device Information
---@field path string
---@field vendor_id number
---@field product_id number
---@field serial_number string?
---@field release_number number
---@field manufacturer_string string?
---@field product_string string?
---@field interface_number number
---@field bus_type string
local deviceInfo = {}

---@class ByteArray: userdata, number[] Byte Array Utility, is userdata containing a byte array
---@overload fun(...: number): ByteArray Creates a new byte table
buf = {}

---Pads left of current data with specified amount of zeros
---@param amount number Amount of zeros to insert at beginning
---@return ByteArray
function buf:padLeft(amount) end

---Pads right of current data with specified amount of zeros
---@param amount number Amount of zeros to append to the end
---@return ByteArray
function buf:padRight(amount) end

---Appends specified bytes to the end of the data
---@param bytes ByteArray Byte array to append
---@return ByteArray
function buf:append(bytes) end

---Appends specified bytes to the end of the data
---@param ... number Bytes to append
---@return ByteArray
function buf:appendBuf(...) end

---Splits byte array into array of byte arrays using size as chunk size
---@param size number
---@return ByteArray[]
function buf:chunks(size) end

---Repeats the byte array by specified amount of times
---@param amount number
---@return ByteArray
function buf:repeated(amount) end

---Destroys the userdata and converts it into a simple table
---@return number[]
function buf:toTable() end

---Converts a table containing bytes into byte array
---@param value table Table to convert
---@return ByteArray
function buf.convert(value) end

---Byte utility that simplifies conversion between signed and unsigned numbers and combining bytes into bigger numbers
byte = {}

---Converts byte to signed 8 bit number
---@param b number
---@return number
function byte.i8(b) end

---Combines bytes into unsigned 16 bit number as little endian
---@param b1 number
---@param b2 number
---@return number
function byte.u16(b1, b2) end

---Combines bytes into unsigned 16 bit number as big endian
---@param b1 number
---@param b2 number
---@return number
function byte.u16be(b1, b2) end

---Combines bytes into signed 16 bit number as little endian
---@param b1 number
---@param b2 number
---@return number
function byte.i16(b1, b2) end

---Combines bytes into signed 16 bit number as big endian
---@param b1 number
---@param b2 number
---@return number
function byte.i16be(b1, b2) end

---Combines bytes into unsigned 32 bit number as little endian
---@param b1 number
---@param b2 number
---@param b3 number
---@param b4 number
---@return number
function byte.u32(b1, b2, b3, b4) end

---Combines bytes into unsigned 32 bit number as big endian
---@param b1 number
---@param b2 number
---@param b3 number
---@param b4 number
---@return number
function byte.u32be(b1, b2, b3, b4) end

---Combines bytes into signed 32 bit number as little endian
---@param b1 number
---@param b2 number
---@param b3 number
---@param b4 number
---@return number
function byte.i32(b1, b2, b3, b4) end

---Combines bytes into signed 32 bit number as big endian
---@param b1 number
---@param b2 number
---@param b3 number
---@param b4 number
---@return number
function byte.i32be(b1, b2, b3, b4) end

---Combines bytes into unsigned 64 bit number as little endian
---@param b1 number
---@param b2 number
---@param b3 number
---@param b4 number
---@param b5 number
---@param b6 number
---@param b7 number
---@param b8 number
---@return number
function byte.u64(b1, b2, b3, b4, b5, b6, b7, b8) end

---Combines bytes into unsigned 64 bit number as big endian
---@param b1 number
---@param b2 number
---@param b3 number
---@param b4 number
---@param b5 number
---@param b6 number
---@param b7 number
---@param b8 number
---@return number
function byte.u64be(b1, b2, b3, b4, b5, b6, b7, b8) end

---Combines bytes into signed 64 bit number as little endian
---@param b1 number
---@param b2 number
---@param b3 number
---@param b4 number
---@param b5 number
---@param b6 number
---@param b7 number
---@param b8 number
---@return number
function byte.i64(b1, b2, b3, b4, b5, b6, b7, b8) end

---Combines bytes into signed 64 bit number as big endian
---@param b1 number
---@param b2 number
---@param b3 number
---@param b4 number
---@param b5 number
---@param b6 number
---@param b7 number
---@param b8 number
---@return number
function byte.i64be(b1, b2, b3, b4, b5, b6, b7, b8) end

---Splits provided number into 2 bytes with little endian
---@param value number
---@return number
---@return number
function byte.as2Bytes(value) end

---Splits provided number into 4 bytes with little endian
---@param value number
---@return number
---@return number
---@return number
---@return number
function byte.as4Bytes(value) end

---Splits provided number into 8 bytes with little endian
---@param value number
---@return number
---@return number
---@return number
---@return number
---@return number
---@return number
---@return number
---@return number
function byte.as8Bytes(value) end

---Splits provided number into 2 bytes with big endian
---@param value number
---@return number
---@return number
function byte.as2BEBytes(value) end

---Splits provided number into 4 bytes with big endian
---@param value number
---@return number
---@return number
---@return number
---@return number
function byte.as4BEBytes(value) end

---Splits provided number into 8 bytes with big endian
---@param value number
---@return number
---@return number
---@return number
---@return number
---@return number
---@return number
---@return number
---@return number
function byte.as8BEBytes(value) end

---@alias ColorType # Pixel color type
---| '"l8"' # Grayscale 8 bit
---| '"la8"' # Grayscale 8 bit with alpha
---| '"rgb8"' # RGB 8 bit
---| '"rgba8"' # RGB 8 bit with alpha
---| '"l16"' # Grayscale 16 bit
---| '"la16"' # Grayscale 16 bit with alpha
---| '"rgb16"' # RGB 16 bit
---| '"rgba16"' # RGB 16 bit with alpha
---| '"rgb32f"' # RGB 32 bit floating points
---| '"rgba32f"' # RGB 32 bit floating points with alpha

---@alias FilterType # Resize filter type
---| '"nearest"' # Nearest sample filter
---| '"linear"' # Linear/Triangle sample filter
---| '"triangle"' # Linear/Triangle sample filter
---| '"cubic"' # Cubic/CatmullRom sample filter
---| '"catmull"' # Cubic/CatmullRom sample filter
---| '"catmullrom"' # Cubic/CatmullRom sample filter
---| '"gaussian"' # Gaussian sample filter
---| '"lanczos"' # Lanczos3 sample filter
---| '"lanczos3"' # Lanczos3 sample filter

---@class DynamicImage Generic container for image data
---@field colorType ColorType Pixel color type of the image
---@field width number Width of the image
---@field height number Height of the image
local imageInstance = {}

---Consumes the image and returns resized version of the image
---@param width number New image width
---@param height number New image height
---@param filter FilterType Pixel filter to use to resize
---@param exact boolean? To disregard aspect ratio or not
---@return DynamicImage
function imageInstance:resized(width, height, filter, exact) end

---Consumes the image and returns resized version of the image that fills the specified dimensions
---@param width number New image width
---@param height number New image height
---@param filter FilterType Pixel filter to use to resize
---@return DynamicImage
function imageInstance:cropResized(width, height, filter, exact) end

---Consumes the image and returns 90 degrees rotated version of the image
---@param ccw boolean? To rotate counter clockwise instead
---@return DynamicImage
function imageInstance:rotated90(ccw) end

---Consumes the image and returns 180 degrees rotated version of the image
---@return DynamicImage
function imageInstance:rotated180() end

---Consumes the image and returns horizontally flipped version of the image
---@return DynamicImage
function imageInstance:hFlipped() end

---Consumes the image and returns vertically flipped version of the image
---@return DynamicImage
function imageInstance:vFlipped() end

---Makes a copy of the image
---@return DynamicImage
function imageInstance:clone() end

---Encodes image into JPEG byte array
---@param quality number
---@return ByteArray
function imageInstance:encodeJpeg(quality) end

---Encodes image into PNG byte array
---@return ByteArray
function imageInstance:encodePng() end

---Encodes image into BMP byte array
---@return ByteArray
function imageInstance:encodeBmp() end

---Consumes the image and returns array of numbers and color type that was used
---@return number[] # Pixel data
---@return ColorType
function imageInstance:toRaw() end

---Consumes the image and returns a converted image based on specified color type
---@param colorType ColorType
---@return DynamicImage
function imageInstance:toColor(colorType) end

---Image utility that allows working with image data
image = {}

---Creates new image with RGB8 pixels
---@param width number
---@param height number
---@return DynamicImage
function image.newRGB8(width, height) end

---Creates new image with RGBA8 pixels
---@param width number
---@param height number
---@return DynamicImage
function image.newRGBA8(width, height) end

---Creates new image with RGB16 pixels
---@param width number
---@param height number
---@return DynamicImage
function image.newRGB16(width, height) end

---Creates new image with RGBA16 pixels
---@param width number
---@param height number
---@return DynamicImage
function image.newRGBA16(width, height) end

---Creates new image with L8 pixels
---@param width number
---@param height number
---@return DynamicImage
function image.newL8(width, height) end

---Creates new image with LA8 pixels
---@param width number
---@param height number
---@return DynamicImage
function image.newLA8(width, height) end

---Creates new image with L16 pixels
---@param width number
---@param height number
---@return DynamicImage
function image.newL16(width, height) end

---Creates new image with LA16 pixels
---@param width number
---@param height number
---@return DynamicImage
function image.newLA16(width, height) end

---Creates new image with RGB32F pixels
---@param width number
---@param height number
---@return DynamicImage
function image.newRGB32F(width, height) end

---Creates new image with RGBA32F pixels
---@param width number
---@param height number
---@return DynamicImage
function image.newRGBA32F(width, height) end