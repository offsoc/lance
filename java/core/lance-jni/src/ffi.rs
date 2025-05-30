// Copyright 2024 Lance Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::slice;

use crate::error::Result;
use crate::utils::{get_index_params, get_query};
use crate::Error;
use jni::objects::{JByteBuffer, JFloatArray, JObjectArray, JString};
use jni::sys::jobjectArray;
use jni::{objects::JObject, JNIEnv};

/// Extend JNIEnv with helper functions.
pub trait JNIEnvExt {
    /// Get integers from Java List<Integer> object.
    fn get_integers(&mut self, obj: &JObject) -> Result<Vec<i32>>;

    /// Get longs from Java List<Long> object.
    fn get_longs(&mut self, obj: &JObject) -> Result<Vec<i64>>;

    /// Get strings from Java List<String> object.
    fn get_strings(&mut self, obj: &JObject) -> Result<Vec<String>>;

    /// Converts a Java `String[]` array to a Rust `Vec<String>`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences a raw pointer `jobjectArray`.
    /// The caller must ensure that the `jobjectArray` is a valid Java string array
    /// and that the JNI environment `self` is correctly initialized and valid.
    /// The function assumes that the `jobjectArray` is not null and that its elements
    /// are valid Java strings. If these conditions are not met, the function may
    /// exhibit undefined behavior.
    #[allow(dead_code)]
    unsafe fn get_strings_array(&mut self, obj: jobjectArray) -> Result<Vec<String>>;

    /// Get Option<String> from Java Optional<String>.
    fn get_string_opt(&mut self, obj: &JObject) -> Result<Option<String>>;

    /// Get Option<Vec<String>> from Java Optional<List<String>>.
    #[allow(dead_code)]
    fn get_strings_opt(&mut self, obj: &JObject) -> Result<Option<Vec<String>>>;

    /// Get Option<i32> from Java Optional<Integer>.
    fn get_int_opt(&mut self, obj: &JObject) -> Result<Option<i32>>;

    /// Get Option<Vec<i32>> from Java Optional<List<Integer>>.
    fn get_ints_opt(&mut self, obj: &JObject) -> Result<Option<Vec<i32>>>;

    /// Get Option<i64> from Java Optional<Long>.
    fn get_long_opt(&mut self, obj: &JObject) -> Result<Option<i64>>;

    /// Get Option<u64> from Java Optional<Long>.
    fn get_u64_opt(&mut self, obj: &JObject) -> Result<Option<u64>>;

    /// Get Option<&[u8]> from Java Optional<ByteBuffer>.
    fn get_bytes_opt(&mut self, obj: &JObject) -> Result<Option<&[u8]>>;

    // Get String from Java Object with given method name.
    fn get_string_from_method(&mut self, obj: &JObject, method_name: &str) -> Result<String>;
    // Get float array from Java Object with given method name.
    fn get_vec_f32_from_method(&mut self, obj: &JObject, method_name: &str) -> Result<Vec<f32>>;
    // Get int as usize from Java Object with given method name.
    fn get_int_as_usize_from_method(&mut self, obj: &JObject, method_name: &str) -> Result<usize>;
    // Get boolean from Java Object with given method name.
    fn get_boolean_from_method(&mut self, obj: &JObject, method_name: &str) -> Result<bool>;
    // Get Option<uszie> from Java Object Optional<Integer> with given method name.
    fn get_optional_usize_from_method(
        &mut self,
        obj: &JObject,
        method_name: &str,
    ) -> Result<Option<usize>>;
    // Get Option<i32> from Java Object Optional<Integer> with given method name.
    fn get_optional_i32_from_method(
        &mut self,
        obj: &JObject,
        method_name: &str,
    ) -> Result<Option<i32>>;
    // Get Option<i32> from Java Object Optional<Integer> with given method name.
    fn get_optional_u32_from_method(
        &mut self,
        obj: &JObject,
        method_name: &str,
    ) -> Result<Option<u32>>;

    fn get_optional_integer_from_method<T>(
        &mut self,
        obj: &JObject,
        method_name: &str,
    ) -> Result<Option<T>>
    where
        T: TryFrom<i32>,
        <T as TryFrom<i32>>::Error: std::fmt::Debug;

    fn get_optional_from_method<T, F>(
        &mut self,
        obj: &JObject,
        method_name: &str,
        f: F,
    ) -> Result<Option<T>>
    where
        F: FnOnce(&mut JNIEnv, JObject) -> Result<T>;

    fn get_optional<T, F>(&mut self, obj: &JObject, f: F) -> Result<Option<T>>
    where
        F: FnOnce(&mut JNIEnv, &JObject) -> Result<T>;
}

impl JNIEnvExt for JNIEnv<'_> {
    fn get_integers(&mut self, obj: &JObject) -> Result<Vec<i32>> {
        let list = self.get_list(obj)?;
        let mut iter = list.iter(self)?;
        let mut results = Vec::with_capacity(list.size(self)? as usize);
        while let Some(elem) = iter.next(self)? {
            let int_obj = self.call_method(elem, "intValue", "()I", &[])?;
            let int_value = int_obj.i()?;
            results.push(int_value);
        }
        Ok(results)
    }

    fn get_longs(&mut self, obj: &JObject) -> Result<Vec<i64>> {
        let list = self.get_list(obj)?;
        let mut iter = list.iter(self)?;
        let mut results = Vec::with_capacity(list.size(self)? as usize);
        while let Some(elem) = iter.next(self)? {
            let long_obj = self.call_method(elem, "longValue", "()J", &[])?;
            let long_value = long_obj.j()?;
            results.push(long_value);
        }
        Ok(results)
    }

    fn get_strings(&mut self, obj: &JObject) -> Result<Vec<String>> {
        let list = self.get_list(obj)?;
        let mut iter = list.iter(self)?;
        let mut results = Vec::with_capacity(list.size(self)? as usize);
        while let Some(elem) = iter.next(self)? {
            let jstr = JString::from(elem);
            let val = self.get_string(&jstr)?;
            results.push(val.to_str()?.to_string())
        }
        Ok(results)
    }

    unsafe fn get_strings_array(&mut self, obj: jobjectArray) -> Result<Vec<String>> {
        let jobject_array = unsafe { JObjectArray::from_raw(obj) };
        let array_len = self.get_array_length(&jobject_array)?;
        let mut res: Vec<String> = Vec::new();
        for i in 0..array_len {
            let item: JString = self.get_object_array_element(&jobject_array, i)?.into();
            res.push(self.get_string(&item)?.into());
        }
        Ok(res)
    }

    fn get_string_opt(&mut self, obj: &JObject) -> Result<Option<String>> {
        self.get_optional(obj, |env, inner_obj| {
            let java_obj_gen = env.call_method(inner_obj, "get", "()Ljava/lang/Object;", &[])?;
            let java_string_obj = java_obj_gen.l()?;
            let jstr = JString::from(java_string_obj);
            let val = env.get_string(&jstr)?;
            Ok(val.to_str()?.to_string())
        })
    }

    fn get_strings_opt(&mut self, obj: &JObject) -> Result<Option<Vec<String>>> {
        self.get_optional(obj, |env, inner_obj| {
            let java_obj_gen = env.call_method(inner_obj, "get", "()Ljava/lang/Object;", &[])?;
            let java_list_obj = java_obj_gen.l()?;
            env.get_strings(&java_list_obj)
        })
    }

    fn get_int_opt(&mut self, obj: &JObject) -> Result<Option<i32>> {
        self.get_optional(obj, |env, inner_obj| {
            let java_obj_gen = env.call_method(inner_obj, "get", "()Ljava/lang/Object;", &[])?;
            let java_int_obj = java_obj_gen.l()?;
            let int_obj = env.call_method(java_int_obj, "intValue", "()I", &[])?;
            let int_value = int_obj.i()?;
            Ok(int_value)
        })
    }

    fn get_ints_opt(&mut self, obj: &JObject) -> Result<Option<Vec<i32>>> {
        self.get_optional(obj, |env, inner_obj| {
            let java_obj_gen = env.call_method(inner_obj, "get", "()Ljava/lang/Object;", &[])?;
            let java_list_obj = java_obj_gen.l()?;
            env.get_integers(&java_list_obj)
        })
    }

    fn get_long_opt(&mut self, obj: &JObject) -> Result<Option<i64>> {
        self.get_optional(obj, |env, inner_obj| {
            let java_obj_gen = env.call_method(inner_obj, "get", "()Ljava/lang/Object;", &[])?;
            let java_long_obj = java_obj_gen.l()?;
            let long_obj = env.call_method(java_long_obj, "longValue", "()J", &[])?;
            let long_value = long_obj.j()?;
            Ok(long_value)
        })
    }

    fn get_u64_opt(&mut self, obj: &JObject) -> Result<Option<u64>> {
        self.get_optional(obj, |env, inner_obj| {
            let java_obj_gen = env.call_method(inner_obj, "get", "()Ljava/lang/Object;", &[])?;
            let java_long_obj = java_obj_gen.l()?;
            let long_obj = env.call_method(java_long_obj, "longValue", "()J", &[])?;
            let long_value = long_obj.j()?;
            Ok(long_value as u64)
        })
    }

    fn get_bytes_opt(&mut self, obj: &JObject) -> Result<Option<&[u8]>> {
        self.get_optional(obj, |env, inner_obj| {
            let java_obj_gen = env.call_method(inner_obj, "get", "()Ljava/lang/Object;", &[])?;
            let java_byte_buffer_obj = java_obj_gen.l()?;
            let j_byte_buffer = JByteBuffer::from(java_byte_buffer_obj);
            let raw_data = env.get_direct_buffer_address(&j_byte_buffer)?;
            let capacity = env.get_direct_buffer_capacity(&j_byte_buffer)?;
            let data = unsafe { slice::from_raw_parts(raw_data, capacity) };
            Ok(data)
        })
    }

    fn get_string_from_method(&mut self, obj: &JObject, method_name: &str) -> Result<String> {
        let string_obj = self
            .call_method(obj, method_name, "()Ljava/lang/String;", &[])?
            .l()?;
        let jstring = JString::from(string_obj);
        let rust_string = self.get_string(&jstring)?.into();
        Ok(rust_string)
    }

    fn get_vec_f32_from_method(&mut self, obj: &JObject, method_name: &str) -> Result<Vec<f32>> {
        let array: JFloatArray = self.call_method(obj, method_name, "()[F", &[])?.l()?.into();
        let length = self.get_array_length(&array)?;
        let mut buffer = vec![0.0f32; length as usize];
        self.get_float_array_region(&array, 0, &mut buffer)?;
        Ok(buffer)
    }

    fn get_int_as_usize_from_method(&mut self, obj: &JObject, method_name: &str) -> Result<usize> {
        Ok(self.call_method(obj, method_name, "()I", &[])?.i()? as usize)
    }

    fn get_boolean_from_method(&mut self, obj: &JObject, method_name: &str) -> Result<bool> {
        Ok(self.call_method(obj, method_name, "()Z", &[])?.z()?)
    }

    fn get_optional_i32_from_method(
        &mut self,
        obj: &JObject,
        method_name: &str,
    ) -> Result<Option<i32>> {
        self.get_optional_integer_from_method(obj, method_name)
    }

    fn get_optional_u32_from_method(
        &mut self,
        obj: &JObject,
        method_name: &str,
    ) -> Result<Option<u32>> {
        self.get_optional_integer_from_method(obj, method_name)
    }

    fn get_optional_usize_from_method(
        &mut self,
        obj: &JObject,
        method_name: &str,
    ) -> Result<Option<usize>> {
        self.get_optional_integer_from_method(obj, method_name)
    }

    fn get_optional_integer_from_method<T>(
        &mut self,
        obj: &JObject,
        method_name: &str,
    ) -> Result<Option<T>>
    where
        T: TryFrom<i32>,
        <T as TryFrom<i32>>::Error: std::fmt::Debug,
    {
        let java_object = self
            .call_method(obj, method_name, "()Ljava/util/Optional;", &[])?
            .l()?;
        let rust_obj = if self
            .call_method(&java_object, "isPresent", "()Z", &[])?
            .z()?
        {
            let inner_jobj = self
                .call_method(&java_object, "get", "()Ljava/lang/Object;", &[])?
                .l()?;
            let inner_value = self.call_method(&inner_jobj, "intValue", "()I", &[])?.i()?;
            Some(T::try_from(inner_value).map_err(|e| {
                Error::io_error(format!("Failed to convert from i32 to rust type: {:?}", e))
            })?)
        } else {
            None
        };
        Ok(rust_obj)
    }

    fn get_optional_from_method<T, F>(
        &mut self,
        obj: &JObject,
        method_name: &str,
        f: F,
    ) -> Result<Option<T>>
    where
        F: FnOnce(&mut JNIEnv, JObject) -> Result<T>,
    {
        let optional_obj = self
            .call_method(obj, method_name, "()Ljava/util/Optional;", &[])?
            .l()?;

        if self
            .call_method(&optional_obj, "isPresent", "()Z", &[])?
            .z()?
        {
            let inner_obj = self
                .call_method(&optional_obj, "get", "()Ljava/lang/Object;", &[])?
                .l()?;
            f(self, inner_obj).map(Some)
        } else {
            Ok(None)
        }
    }

    fn get_optional<T, F>(&mut self, obj: &JObject, f: F) -> Result<Option<T>>
    where
        F: FnOnce(&mut JNIEnv, &JObject) -> Result<T>,
    {
        if obj.is_null() {
            return Ok(None);
        }
        let is_present = self.call_method(obj, "isPresent", "()Z", &[])?;
        if is_present.z()? {
            f(self, obj).map(Some)
        } else {
            // TODO(lu): put get java object into here cuz can only get java Object
            Ok(None)
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_lancedb_lance_test_JniTestHelper_parseInts(
    mut env: JNIEnv,
    _obj: JObject,
    list_obj: JObject, // List<Integer>
) {
    ok_or_throw_without_return!(env, env.get_integers(&list_obj));
}

#[no_mangle]
pub extern "system" fn Java_com_lancedb_lance_test_JniTestHelper_parseLongs(
    mut env: JNIEnv,
    _obj: JObject,
    list_obj: JObject, // List<Long>
) {
    ok_or_throw_without_return!(env, env.get_longs(&list_obj));
}

#[no_mangle]
pub extern "system" fn Java_com_lancedb_lance_test_JniTestHelper_parseIntsOpt(
    mut env: JNIEnv,
    _obj: JObject,
    list_obj: JObject, // Optional<List<Integer>>
) {
    ok_or_throw_without_return!(env, env.get_ints_opt(&list_obj));
}

#[no_mangle]
pub extern "system" fn Java_com_lancedb_lance_test_JniTestHelper_parseQuery(
    mut env: JNIEnv,
    _obj: JObject,
    query_opt: JObject, // Optional<TmpQuery>
) {
    ok_or_throw_without_return!(env, get_query(&mut env, query_opt));
}

#[no_mangle]
pub extern "system" fn Java_com_lancedb_lance_test_JniTestHelper_parseIndexParams(
    mut env: JNIEnv,
    _obj: JObject,
    index_params_obj: JObject, // IndexParams
) {
    ok_or_throw_without_return!(env, get_index_params(&mut env, index_params_obj));
}
