namespace ElgatoStreamDeck;

public readonly struct ImageMode {
	public ImageFormat Mode { get; init; }
	public (uint, uint) Resolution { get; init; }
	public ImageRotation Rotation { get; init; }
	public ImageMirroring Mirror { get; init; }
}

public enum ImageFormat {
	None,
	Bmp,
	Jpeg
}

public enum ImageRotation {
	Rot0,
	Rot90,
	Rot180,
	Rot270
}

public enum ImageMirroring {
	None,
	X,
	Y,
	Both
}