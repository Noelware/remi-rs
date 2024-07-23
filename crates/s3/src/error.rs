// 🐻‍❄️🧶 remi-rs: Robust, and simple asynchronous Rust crate to handle storage-related communications with different storage providers
// Copyright (c) 2022-2024 Noelware, LLC. <team@noelware.org>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// this is not available due to issues (so far), might resolve in the future

use aws_sdk_s3::{
    operation::{
        create_bucket::CreateBucketError, delete_object::DeleteObjectError, get_object::GetObjectError,
        head_bucket::HeadBucketError, head_object::HeadObjectError, list_buckets::ListBucketsError,
        list_objects_v2::ListObjectsV2Error, put_object::PutObjectError,
    },
    primitives::SdkBody,
};
use aws_smithy_runtime_api::{
    client::result::SdkError,
    client::result::{ConstructionFailure, DispatchFailure, ResponseError, TimeoutError},
    http::Response,
};
use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

/// Type alias for [`std::result::Result`]<`T`, [`Error`]>.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn lib<T: Into<Cow<'static, str>>>(msg: T) -> Error {
    Error::Library(msg.into())
}

/// Represents a generalised error that inlines all service errors and uses [`Response`]<[`SdkBody`]>
/// as the response type.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Request failed during construction, it was not dispatched over the network.
    ConstructionFailure(ConstructionFailure),

    /// Request failed due to a timeout, the request MAY have been sent and received.
    TimeoutError(TimeoutError),

    /// Request failed during dispatch, an HTTP response was not received. The request MAY
    /// have been set.
    DispatchFailure(DispatchFailure),

    /// A response was received but it was not parseable according to the protocol. (for example, the
    /// server hung up without sending a complete response)
    Response(ResponseError<Response<SdkBody>>),

    /// Amazon S3 was unable to list buckets. This happens when you call [`StorageService::init`][crate::StorageService::init],
    /// since the library performs checks whenever if the bucket exists or not and it needs the ability to check.
    ListBuckets(ListBucketsError),

    /// Amazon S3 was unable to create the bucket for some reason, this will never hit the
    /// [`CreateBucketError::BucketAlreadyExists`] or [`CreateBucketError::BucketAlreadyOwnedByYou`]
    /// variants, something might've been unhandled and is probably isn't your fault.
    ///
    /// * this would be thrown from the [`StorageService::init`][remi::StorageService::init]
    /// trait method
    CreateBucket(CreateBucketError),

    /// Amazon S3 was unable to get the object that you were looking for either
    /// from the [`StorageService::open`][remi::StorageService::open] or the
    /// [`StorageService::blob`][remi::StorageService::blob] methods.
    ///
    /// The [`GetObjectError::NoSuchKey`] variant will never be reached since
    /// it'll return `Ok(None)` if the key wasn't present in S3, this might
    /// result in an invalid object state ([`GetObjectError::InvalidObjectState`])
    /// or an unhandled variant that the Rust SDK doesn't support *yet*.
    ///
    /// * this would be thrown from the [`StorageService::open`][remi::StorageService::open]
    /// or the [`StorageService::blob`][remi::StorageService::blob] trait methods.
    GetObject(GetObjectError),

    /// Amazon S3 was unable to list objects from the specific requirements that
    /// it was told to list objects from a [`ListBlobsRequest`][remi::ListBlobsRequest].
    ///
    /// This might be in a unhandled state as [`ListObjectsV2Error::NoSuchBucket`] should never
    /// be matched since `remi-s3` handles creating buckets if they don't exist when
    /// [`StorageService::init`][remi::StorageService::int] is called.
    ///
    /// * this would be thrown from the [`StorageService::open`][remi::StorageService::open]
    /// or the [`StorageService::blob`][remi::StorageService::blob] trait methods.
    ListObjectsV2(ListObjectsV2Error),

    /// Amazon S3 was unable to delete an object from the service.
    ///
    /// * this would be thrown from the [`StorageService::delete`][remi::StorageService::delete] trait method.
    DeleteObject(DeleteObjectError),

    /// Amazon S3 was unable to check the existence of an object. This will never
    /// reach the [`HeadObjectError::NotFound`] state as it'll return `Ok(false)`.
    ///
    /// * this would be thrown from the [`StorageService::exists`][remi::StorageService::exists] trait method.
    HeadObject(HeadObjectError),

    /// Amazon S3 was unable to put an object into the service.
    ///
    /// * this would be thrown from the [`StorageService::upload`][remi::StorageService::upload] trait method.
    PutObject(PutObjectError),

    /// Occurs when an error occurred when transforming AWS S3's responses.
    ByteStream(aws_sdk_s3::primitives::ByteStreamError),

    /// Occurs when `remi-s3` cannot perform a HEAD request to the current bucket. This is mainly
    /// used in healthchecks to determine if the storage service is ok.
    HeadBucket(HeadBucketError),

    /// Something that `remi-s3` has emitted on its own.
    Library(Cow<'static, str>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error as E;

        match self {
            E::ByteStream(err) => Display::fmt(err, f),
            E::Response(_) => f.write_str("response was received but it was not parseable according to the protocol."),
            E::TimeoutError(_) => {
                f.write_str("request failed due to a timeout, the request MAY have been sent and received.")
            }

            E::ConstructionFailure(_) => {
                f.write_str("request failed during construction, it was not dispatched over the network.")
            }

            E::DispatchFailure(_) => f.write_str(
                "request failed during dispatch, an HTTP response was not received. the request MAY have been set.",
            ),

            E::CreateBucket(err) => Display::fmt(err, f),
            E::DeleteObject(err) => Display::fmt(err, f),
            E::GetObject(err) => Display::fmt(err, f),
            E::HeadObject(err) => Display::fmt(err, f),
            E::ListBuckets(err) => Display::fmt(err, f),
            E::ListObjectsV2(err) => Display::fmt(err, f),
            E::PutObject(err) => Display::fmt(err, f),
            E::HeadBucket(err) => Display::fmt(err, f),
            E::Library(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<SdkError<ListBucketsError, Response<SdkBody>>> for Error {
    fn from(error: SdkError<ListBucketsError, Response<SdkBody>>) -> Self {
        match error {
            SdkError::ConstructionFailure(err) => Self::ConstructionFailure(err),
            SdkError::DispatchFailure(err) => Self::DispatchFailure(err),
            SdkError::TimeoutError(err) => Self::TimeoutError(err),
            SdkError::ResponseError(err) => Self::Response(err),
            err => Error::ListBuckets(err.into_service_error()),
        }
    }
}

impl From<SdkError<CreateBucketError, Response<SdkBody>>> for Error {
    fn from(error: SdkError<CreateBucketError, Response<SdkBody>>) -> Self {
        match error {
            SdkError::ConstructionFailure(err) => Self::ConstructionFailure(err),
            SdkError::DispatchFailure(err) => Self::DispatchFailure(err),
            SdkError::TimeoutError(err) => Self::TimeoutError(err),
            SdkError::ResponseError(err) => Self::Response(err),
            err => Error::CreateBucket(err.into_service_error()),
        }
    }
}

impl From<SdkError<GetObjectError, Response<SdkBody>>> for Error {
    fn from(error: SdkError<GetObjectError, Response<SdkBody>>) -> Self {
        match error {
            SdkError::ConstructionFailure(err) => Self::ConstructionFailure(err),
            SdkError::DispatchFailure(err) => Self::DispatchFailure(err),
            SdkError::TimeoutError(err) => Self::TimeoutError(err),
            SdkError::ResponseError(err) => Self::Response(err),
            err => Error::GetObject(err.into_service_error()),
        }
    }
}

impl From<GetObjectError> for Error {
    fn from(value: GetObjectError) -> Self {
        Self::GetObject(value)
    }
}

impl From<SdkError<ListObjectsV2Error, Response<SdkBody>>> for Error {
    fn from(error: SdkError<ListObjectsV2Error, Response<SdkBody>>) -> Self {
        match error {
            SdkError::ConstructionFailure(err) => Self::ConstructionFailure(err),
            SdkError::DispatchFailure(err) => Self::DispatchFailure(err),
            SdkError::TimeoutError(err) => Self::TimeoutError(err),
            SdkError::ResponseError(err) => Self::Response(err),
            err => Error::ListObjectsV2(err.into_service_error()),
        }
    }
}

impl From<SdkError<DeleteObjectError, Response<SdkBody>>> for Error {
    fn from(error: SdkError<DeleteObjectError, Response<SdkBody>>) -> Self {
        match error {
            SdkError::ConstructionFailure(err) => Self::ConstructionFailure(err),
            SdkError::DispatchFailure(err) => Self::DispatchFailure(err),
            SdkError::TimeoutError(err) => Self::TimeoutError(err),
            SdkError::ResponseError(err) => Self::Response(err),
            err => Error::DeleteObject(err.into_service_error()),
        }
    }
}

impl From<SdkError<HeadObjectError, Response<SdkBody>>> for Error {
    fn from(error: SdkError<HeadObjectError, Response<SdkBody>>) -> Self {
        match error {
            SdkError::ConstructionFailure(err) => Self::ConstructionFailure(err),
            SdkError::DispatchFailure(err) => Self::DispatchFailure(err),
            SdkError::TimeoutError(err) => Self::TimeoutError(err),
            SdkError::ResponseError(err) => Self::Response(err),
            err => Error::HeadObject(err.into_service_error()),
        }
    }
}

impl From<HeadObjectError> for Error {
    fn from(value: HeadObjectError) -> Self {
        Self::HeadObject(value)
    }
}

impl From<SdkError<PutObjectError, Response<SdkBody>>> for Error {
    fn from(error: SdkError<PutObjectError, Response<SdkBody>>) -> Self {
        match error {
            SdkError::ConstructionFailure(err) => Self::ConstructionFailure(err),
            SdkError::DispatchFailure(err) => Self::DispatchFailure(err),
            SdkError::TimeoutError(err) => Self::TimeoutError(err),
            SdkError::ResponseError(err) => Self::Response(err),
            err => Error::PutObject(err.into_service_error()),
        }
    }
}

impl From<SdkError<HeadBucketError, Response<SdkBody>>> for Error {
    fn from(value: SdkError<HeadBucketError, Response<SdkBody>>) -> Self {
        match value {
            SdkError::ConstructionFailure(err) => Self::ConstructionFailure(err),
            SdkError::DispatchFailure(err) => Self::DispatchFailure(err),
            SdkError::TimeoutError(err) => Self::TimeoutError(err),
            SdkError::ResponseError(err) => Self::Response(err),
            err => Error::HeadBucket(err.into_service_error()),
        }
    }
}

impl From<aws_sdk_s3::primitives::ByteStreamError> for Error {
    fn from(value: aws_sdk_s3::primitives::ByteStreamError) -> Self {
        Self::ByteStream(value)
    }
}
