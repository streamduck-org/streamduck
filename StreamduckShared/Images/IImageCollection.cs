// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using SixLabors.ImageSharp;

namespace Streamduck.Images;

public interface IImageCollection {
	public IReadOnlyDictionary<Guid, Image> Images { get; }
	public Guid Add(Image image);
	public Image? Delete(Guid id);
}