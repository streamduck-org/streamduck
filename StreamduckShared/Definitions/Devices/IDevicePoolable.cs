using System.Threading.Tasks;

namespace Streamduck.Definitions.Devices;

/**
 * Device should implement this interface if it needs to be pooled to receive events from it.
 * You don't have to implement this if your device is event based or it can't generate any events.
 */
public interface IDevicePoolable {
	Task Poll();
}