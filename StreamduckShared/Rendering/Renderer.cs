using System;
using SixLabors.ImageSharp;
using Streamduck.Api;
using Streamduck.Cores;

namespace Streamduck.Rendering; 

/**
 * Renderer that will render images for input's screens
 */
public abstract class Renderer : INamed {
	public abstract long Hash(ScreenItem input, object renderConfig);
	public abstract Image Render(ScreenItem input, object renderConfig);
	public abstract string Name { get; }
}

public abstract class Renderer<T> : Renderer where T : class {
	public override long Hash(ScreenItem input, object renderConfig) {
		if (renderConfig is not T castedConfig) 
			throw new ArgumentException("Render config was of incorrect type");
		return Hash(input, castedConfig);
	}

	public override Image Render(ScreenItem input, object renderConfig) {
		if (renderConfig is not T castedConfig) 
			throw new ArgumentException("Render config was of incorrect type");
		return Render(input, castedConfig);
	}

	public abstract long Hash(ScreenItem input, T renderConfig);
	public abstract Image Render(ScreenItem input, T renderConfig);
}