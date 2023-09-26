using Avalonia.ReactiveUI;
using Streamduck.UI.ViewModels.DeviceEditor;

namespace Streamduck.UI.Views.DeviceEditor;

public partial class DeviceEditorView : ReactiveUserControl<DeviceEditorViewModel> {
	public DeviceEditorView() {
		InitializeComponent();
	}
}