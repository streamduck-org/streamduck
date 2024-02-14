// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using Streamduck.Inputs;

namespace Streamduck.UI.Design;

public class ShellInput(int x, int y, uint w, uint h, InputIcon icon) : Input(x, y, w, h, icon);