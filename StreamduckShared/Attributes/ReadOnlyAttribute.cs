// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;

namespace Streamduck.Attributes;

/**
 * Make property read only in UI despite having public setter
 */
[AttributeUsage(AttributeTargets.Property)]
public class ReadOnlyAttribute : Attribute { }