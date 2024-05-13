using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Cores;
using Streamduck.Devices;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests.Devices;

[AutoAdd]
public class PushNewEmptyScreen : SocketRequest<PushNewEmptyScreen.Request> {
	public class Request {
		public required NamespacedDeviceIdentifier Identifier { get; set; }
	}

	public override string Name => "Push New Empty Screen";
	public override Task Received(SocketRequester request, Request data) {
		if (!SRUtil.GetDevice(request, data.Identifier, out var device)) return Task.CompletedTask;
		
		device.PushScreen(device.NewScreen());
		request.SendBack(null);
		return Task.CompletedTask;
	}
}