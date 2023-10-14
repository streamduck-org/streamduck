namespace Streamduck.Data;

public struct Int2 {
	public Int2(int x, int y) {
		X = x;
		Y = y;
	}

	public int X { get; set; }
	public int Y { get; set; }

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