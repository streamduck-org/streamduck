using System;

namespace Streamduck.Devices;

public class DeviceDisconnectedException : Exception {
	public override string Message => "Device got disconnected";
}