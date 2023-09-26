using System.Reactive;
using ReactiveUI;
using Streamduck.Devices;

namespace Streamduck.UI.ViewModels.DeviceEditor;

public class DeviceEditorViewModel : ViewModelBase, IRoutableViewModel {
	private readonly NamespacedDeviceIdentifier _deviceName;

	public DeviceEditorViewModel(IScreen hostScreen, NamespacedDeviceIdentifier deviceName) {
		HostScreen = hostScreen;
		_deviceName = deviceName;
	}

	public ReactiveCommand<Unit, IRoutableViewModel?> GoBack => HostScreen.Router.NavigateBack;
	public string Identifier => _deviceName.Identifier;
	public string Description => _deviceName.Description;

	public string UrlPathSegment => "editor";
	public IScreen HostScreen { get; }
}