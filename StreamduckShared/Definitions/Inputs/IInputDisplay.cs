using System.Threading.Tasks;

namespace Streamduck.Definitions.Inputs;

public interface IInputDisplay {
	UInt2 DisplayResolution { get; }
	Task ApplyImage(int key);
	Task ClearImage();
}