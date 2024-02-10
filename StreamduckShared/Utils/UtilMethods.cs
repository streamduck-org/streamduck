// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Linq;

namespace Streamduck.Utils;

public static class UtilMethods {
	public static string FormatAsWords(this string name) => string.Concat(name
		.Select((x, i) => i == 0 ? $"{char.ToUpper(x)}" : char.IsUpper(x) ? $" {x}" : $"{x}")).TrimStart(' ');

	public static T? WeakToNullable<T>(this WeakReference<T> weakReference) where T : class {
		weakReference.TryGetTarget(out var result);
		return result;
	}
}