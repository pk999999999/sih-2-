import time
from pathlib import Path

import boto3
from botocore.exceptions import BotoCoreError, ClientError

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
    for attempt in range(10):
        try:
            try:
                client.head_bucket(Bucket=config.MINIO_BUCKET)
            except ClientError as exc:
                if exc.response["Error"]["Code"] not in {"404", "NoSuchBucket"}:
                    raise
                client.create_bucket(Bucket=config.MINIO_BUCKET)
            client.put_object(Bucket=config.MINIO_BUCKET, Key=key, Body=data, ContentType="application/json")
            return
        except (ClientError, BotoCoreError):
            if attempt == 9:
                raise
            time.sleep(1)


def get_object(key: str) -> bytes:
    if config.STORAGE_BACKEND == "local":
        return (Path(config.EVIDENCE_DIR) / key).read_bytes()
    return _client().get_object(Bucket=config.MINIO_BUCKET, Key=key)["Body"].read()
