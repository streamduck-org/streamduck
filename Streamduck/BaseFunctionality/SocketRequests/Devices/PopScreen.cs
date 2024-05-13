using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Devices;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests.Devices;

[AutoAdd]
public class PopScreen : SocketRequest<PopScreen.Request> {
	public class Request {
		public required NamespacedDeviceIdentifier Identifier { get; set; }
	}


	public override string Name => "Pop Screen";
	public override Task Received(SocketRequester request, Request data) {
		if (!SRUtil.GetDevice(request, data.Identifier, out var device)) return Task.CompletedTask;

		request.SendBack(device.PopScreen() is not null);
		return Task.CompletedTask;
	}
}