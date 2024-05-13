using System.Threading.Tasks;
using Streamduck.Cores;
using Streamduck.Devices;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests;

public static class SRUtil {
	public static bool GetDevice(SocketRequester request, NamespacedDeviceIdentifier identifier, out Core device) {
		if (App.CurrentInstance!.ConnectedDevices.TryGetValue(identifier, out device!)) return true;
		request.SendBackError("Device is not connected or doesn't exist");
		return false;
	}
}