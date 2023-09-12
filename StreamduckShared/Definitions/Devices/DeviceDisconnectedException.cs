using System;

namespace Streamduck.Definitions.Devices; 

public class DeviceDisconnectedException : Exception {
	public override string Message => "Device got disconnected";
}