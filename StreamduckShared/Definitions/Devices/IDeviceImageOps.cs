using System.Threading.Tasks;
using SixLabors.ImageSharp;

namespace Streamduck.Definitions.Devices;

public interface IDeviceImageOps {
	ValueTask<bool> HashExists(int key);
	Task UploadImage(int key, Image image);
}