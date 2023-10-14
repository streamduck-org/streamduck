namespace Streamduck.Data;

public struct Double2 {
	public Double2(double x, double y) {
		X = x;
		Y = y;
	}

	public double X { get; set; }
	public double Y { get; set; }
	
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