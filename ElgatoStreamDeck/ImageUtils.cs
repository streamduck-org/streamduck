// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.IO;
using System.Threading.Tasks;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.Formats.Jpeg;
using SixLabors.ImageSharp.PixelFormats;
using SixLabors.ImageSharp.Processing;
using SixLabors.ImageSharp.Processing.Processors.Transforms;

namespace ElgatoStreamDeck;

public static class ImageUtils {
	private static void MutateImageForDevice(Image<Rgb24> image, ImageMode mode) {
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
				_ => throw new ArgumentOutOfRangeException(nameof(mode))
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
					throw new ArgumentOutOfRangeException(nameof(mode));
			}
		});
	}

	public static byte[] EncodeImageForButton(Image<Rgb24> image, Kind kind, int jpegQuality = 94) {
		var mode = kind.KeyImageMode();

		if (mode.Mode == ImageFormat.None) return Array.Empty<byte>();

		MutateImageForDevice(image, mode);

		using var buffer = new MemoryStream();

		switch (mode.Mode) {
			case ImageFormat.Bmp:
				image.SaveAsBmp(buffer);
				break;
			case ImageFormat.Jpeg:
				image.Save(buffer, new JpegEncoder {
					Quality = jpegQuality
				});
				break;
			case ImageFormat.None:
			default:
				throw new ArgumentOutOfRangeException(nameof(kind));
		}

		return buffer.ToArray();
	}

	public static byte[] EncodeImageForButton(Image image, Kind kind, int jpegQuality = 94) =>
		EncodeImageForButton(image.CloneAs<Rgb24>(), kind, jpegQuality);

	/**
	 * Pixels must be expressed as 3 bytes of Red, Green and Blue
	 */
	public static byte[] EncodeImageForButton(ReadOnlySpan<byte> image, int width, int height, Kind kind,
		int jpegQuality = 94) => EncodeImageForButton(
		Image.LoadPixelData<Rgb24>(image, width, height), kind, jpegQuality
	);

	public static async Task<byte[]> EncodeImageForButtonAsync(Image<Rgb24> image, Kind kind, int jpegQuality = 94) {
		var mode = kind.KeyImageMode();

		if (mode.Mode == ImageFormat.None) return Array.Empty<byte>();

		MutateImageForDevice(image, mode);

		using var buffer = new MemoryStream();

		switch (mode.Mode) {
			case ImageFormat.Bmp:
				await image.SaveAsBmpAsync(buffer);
				break;
			case ImageFormat.Jpeg:
				await image.SaveAsync(buffer, new JpegEncoder {
					Quality = jpegQuality
				});
				break;
			case ImageFormat.None:
			default:
				throw new ArgumentOutOfRangeException(nameof(kind));
		}

		return buffer.ToArray();
	}

	public static async Task<byte[]> EncodeImageForButtonAsync(Image image, Kind kind, int jpegQuality = 94) =>
		await EncodeImageForButtonAsync(image.CloneAs<Rgb24>(), kind, jpegQuality);

	public static byte[] EncodeImageForLcd(Image<Rgb24> image, int targetWidth, int targetHeight,
		int jpegQuality = 94) {
		image.Mutate(i => {
			i.Resize(new ResizeOptions {
				Mode = ResizeMode.Crop,
				Size = new Size(targetWidth, targetHeight),
				Sampler = new BicubicResampler()
			});
		});

		using var buffer = new MemoryStream();

		image.Save(buffer, new JpegEncoder {
			Quality = jpegQuality
		});

		return buffer.ToArray();
	}

	public static byte[] EncodeImageForLcd(Image image, int targetWidth, int targetHeight, int jpegQuality = 94) =>
		EncodeImageForLcd(
			image.CloneAs<Rgb24>(), targetWidth, targetHeight, jpegQuality
		);

	public static byte[] EncodeImageForLcd(ReadOnlySpan<byte> image, int width, int height, int targetWidth,
		int targetHeight, int jpegQuality = 94) => EncodeImageForLcd(
		Image.LoadPixelData<Rgb24>(image, width, height), targetWidth, targetHeight, jpegQuality
	);

	public static async Task<byte[]> EncodeImageForLcdAsync(Image<Rgb24> image, int targetWidth, int targetHeight,
		int jpegQuality = 94) {
		image.Mutate(i => {
			i.Resize(new ResizeOptions {
				Mode = ResizeMode.Crop,
				Size = new Size(targetWidth, targetHeight),
				Sampler = new BicubicResampler()
			});
		});

		using var buffer = new MemoryStream();

		await image.SaveAsync(buffer, new JpegEncoder {
			Quality = jpegQuality
		});

		return buffer.ToArray();
	}

	public static async Task<byte[]> EncodeImageForLcdAsync(Image image, int targetWidth, int targetHeight,
		int jpegQuality = 94) =>
		await EncodeImageForLcdAsync(
			image.CloneAs<Rgb24>(), targetWidth, targetHeight, jpegQuality
		);
}