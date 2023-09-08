namespace Streamduck.Definitions;

public struct UInt2 {
	public UInt2(uint x, uint y) {
		X = x;
		Y = y;
	}

	public uint X { get; set; }
	public uint Y { get; set; }
}