// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;

namespace Streamduck.Attributes;

/**
 * Include non-public property in UI
 */
[AttributeUsage(AttributeTargets.Property)]
public class IncludeAttribute(bool write = false) : Attribute {
	public bool WriteAllowed { get; } = write;
}