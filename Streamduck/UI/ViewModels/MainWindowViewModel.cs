using System.Linq;
using Avalonia;
using DynamicData;
using Streamduck.UI.ViewModels.DeviceList;

namespace Streamduck.UI.ViewModels; 

public class MainWindowViewModel : ViewModelBase {
	public DeviceListViewModel DeviceList { get; } = new();
}