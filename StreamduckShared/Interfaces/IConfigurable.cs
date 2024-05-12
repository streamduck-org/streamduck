// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using Streamduck.Fields;

namespace Streamduck.Interfaces;

/* TODO: Put this information somewhere in Field documentation
 * Options objects will be queried using reflection for public properties and/or attributed properties.
 * - Property that has getter and setter can be edited in UI
 * - Property that only has getter will no be able to be edited in UI and will be shown as read-only
 * - Non-public property annotated with [Include] will be shown in UI and will be read-only unless write parameter is used in [Include]
 * - Non-public properties not annotated with [Include] will not be shown in UI
 *
 * Property field type will be assumed from it's Type, use any of the [Field] attributes to force field type
 */

/**
 * Object provided as Options needs to support serialization,
 * along with having fields that can be represented as Field objects
 */
public interface IConfigurable {
    IEnumerable<Field> Config { get; }
}

/**
 * <p>
 *     Allows types to contain options that can be configured from UI,
 *     saving and loading will be done automatically.
 * </p>
 * <p>
 *     Type provided as Options needs to support serialization and deserialization,
 *     along with having properties that can be represented as Field objects.
 * </p>
 */
public interface IConfigurable<T> : IConfigurable where T : class, new() {
    new T Config { get; set; }
    IEnumerable<Field> IConfigurable.Config => FieldReflector.AnalyzeObject(Config);
}