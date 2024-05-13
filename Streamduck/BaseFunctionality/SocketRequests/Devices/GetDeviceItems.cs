using System.Linq;
using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Cores;
using Streamduck.Devices;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests.Devices;

[AutoAdd]
public class GetDeviceItems : SocketRequest<GetDeviceItems.Request> {
	public class Request {
		public required NamespacedDeviceIdentifier Identifier { get; set; }
		public bool GetPreviews { get; set; } = false;
	}

	public class PartialScreenItemData(ScreenItem item) {
		public bool Renderable { get; set; } = item is ScreenItem.IRenderable;
		public string? Base64JPG { get; set; }
	}

	public override string Name => "Get Device Items";

	public override Task Received(SocketRequester request, Request data) {
		if (!SRUtil.GetDevice(request, data.Identifier, out var device)) return Task.CompletedTask;

		request.SendBack(
			device.CurrentScreen?.Items.Select(
				i => i is not null
					? new PartialScreenItemData(i)
					: null
			)
		);
		return Task.CompletedTask;
	}
}