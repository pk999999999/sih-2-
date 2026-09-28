import time
from pathlib import Path

import boto3
from botocore.exceptions import ClientError

from . import config


def _client():
    return boto3.client(
        "s3",
        endpoint_url=f"http://{config.MINIO_ENDPOINT}",
        aws_access_key_id=config.MINIO_ACCESS_KEY,
        aws_secret_access_key=config.MINIO_SECRET_KEY,
        region_name="us-east-1",
    )


def put_object(key: str, data: bytes) -> None:
    if config.STORAGE_BACKEND == "local":
        path = Path(config.EVIDENCE_DIR) / key
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return
    client = _client()
    for attempt in range(5):
        try:
            client.head_bucket(Bucket=config.MINIO_BUCKET)
            break
        except ClientError as exc:
            if exc.response["Error"]["Code"] in {"404", "NoSuchBucket"}:
                try:
                    client.create_bucket(Bucket=config.MINIO_BUCKET)
                    break
                except ClientError:
                    pass
            if attempt == 4:
                raise
            time.sleep(1)
    client.put_object(Bucket=config.MINIO_BUCKET, Key=key, Body=data, ContentType="application/json")


def get_object(key: str) -> bytes:
    if config.STORAGE_BACKEND == "local":
        return (Path(config.EVIDENCE_DIR) / key).read_bytes()
    return _client().get_object(Bucket=config.MINIO_BUCKET, Key=key)["Body"].read()
