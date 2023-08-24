using System;
using System.IO;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.Formats.Jpeg;
using SixLabors.ImageSharp.PixelFormats;
using SixLabors.ImageSharp.Processing;
using SixLabors.ImageSharp.Processing.Processors.Transforms;

namespace ElgatoStreamDeck;

public static class ImageUtils {
	public static byte[] ConvertImage(Image<Rgb24> image, Kind kind) {
		var mode = kind.KeyImageMode();

		if (mode.Mode == ImageFormat.None) return Array.Empty<byte>();

		image.Mutate(i => {
			i.Resize(new ResizeOptions {
				Mode = ResizeMode.Crop,
				Size = new Size((int)mode.Resolution.Item1, (int)mode.Resolution.Item2),
				Sampler = new BicubicResampler()
			});

			i.Rotate(mode.Rotation switch {
				ImageRotation.Rot0 => RotateMode.None,
				ImageRotation.Rot90 => RotateMode.Rotate90,
				ImageRotation.Rot180 => RotateMode.Rotate180,
				ImageRotation.Rot270 => RotateMode.Rotate270,
				_ => throw new ArgumentOutOfRangeException(nameof(kind))
			});

			switch (mode.Mirror) {
				case ImageMirroring.None:
					break;
				case ImageMirroring.X:
					i.Flip(FlipMode.Horizontal);
					break;
				case ImageMirroring.Y:
					i.Flip(FlipMode.Vertical);
					break;
				case ImageMirroring.Both:
					i.Flip(FlipMode.Horizontal);
					i.Flip(FlipMode.Vertical);
					break;
				default:
					throw new ArgumentOutOfRangeException(nameof(kind));
			}
		});

		using var buffer = new MemoryStream();

		switch (mode.Mode) {
			case ImageFormat.Bmp:
				image.SaveAsBmp(buffer);
				break;
			case ImageFormat.Jpeg:
				image.Save(buffer, new JpegEncoder {
					Quality = 94
				});
				break;
			case ImageFormat.None:
			default:
				throw new ArgumentOutOfRangeException(nameof(kind));
		}

		return buffer.ToArray();
	}

	public static byte[] ConvertImage(Image image, Kind kind) => ConvertImage(image.CloneAs<Rgb24>(), kind);

	/**
	 * Pixels must be expressed as 3 bytes of Red, Green and Blue
	 */
	public static byte[] ConvertImage(byte[] image, int width, int height, Kind kind) => ConvertImage(
		Image.LoadPixelData<Rgb24>(image, width, height), kind
	);
}