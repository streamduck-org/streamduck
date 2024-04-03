// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using Streamduck.Inputs;

namespace Streamduck.Cores;

/**
 * Screen that can contain screen items
 */
public abstract class Screen {
	public abstract Core AssociatedCore { get; }
	public bool CanWrite { get; init; } = true;
	public abstract IReadOnlyCollection<ScreenItem?> Items { get; }
	public abstract ScreenItem CreateItem(int index);
	public abstract ScreenItem DeleteItem(int index);
}