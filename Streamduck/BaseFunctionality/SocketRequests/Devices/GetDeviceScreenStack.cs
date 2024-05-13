using System.Linq;
using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Devices;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests.Devices;

[AutoAdd]
public class GetDeviceScreenStack : SocketRequest<GetDeviceScreenStack.Request> {
	public class Request {
		public required NamespacedDeviceIdentifier Identifier { get; set; }
	}

	public override string Name => "Get Device Screen Stack";
	public override Task Received(SocketRequester request, Request data) {
		if (!SRUtil.GetDevice(request, data.Identifier, out var device)) return Task.CompletedTask;
		
		request.SendBack(device.ScreenStack.Select(s => s.Name).Reverse());
		return Task.CompletedTask;
	}
}