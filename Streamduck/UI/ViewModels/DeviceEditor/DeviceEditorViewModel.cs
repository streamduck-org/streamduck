using System.Reactive;
using ReactiveUI;
using Streamduck.Devices;

namespace Streamduck.UI.ViewModels.DeviceEditor;

public class DeviceEditorViewModel(IScreen hostScreen, NamespacedDeviceIdentifier deviceName) : ViewModelBase,
	IRoutableViewModel {
	public ReactiveCommand<Unit, IRoutableViewModel?> GoBack => HostScreen.Router.NavigateBack;
	public string Identifier => deviceName.Identifier;
	public string Description => deviceName.Description;

	public string UrlPathSegment => "editor";
	public IScreen HostScreen { get; } = hostScreen;
}