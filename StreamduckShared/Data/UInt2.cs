namespace Streamduck.Data;

public struct UInt2 {
	public UInt2(uint x, uint y) {
		X = x;
		Y = y;
	}

	public uint X { get; set; }
	public uint Y { get; set; }
	
	public override string ToString() => $"{{ X: {X}, Y: {Y} }}";
	
	public override int GetHashCode() {
		unchecked {
			var hash = 17;
			hash = hash * 23 + X.GetHashCode();
			hash = hash * 23 + Y.GetHashCode();
			return hash;
		}
	}
}