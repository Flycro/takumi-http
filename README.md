# takumi-http

HTTP server for [Takumi](https://github.com/kane50613/takumi) image rendering.

This project provides a drop-in Docker image for convenient server-side image generation, making it easy to integrate Takumi into any backend (Laravel, Node.js, Go, etc.) via simple HTTP requests.

## Node Tree API

For documentation on the node tree schema, available node types, and styling options, see the [Takumi Reference](https://takumi.kane.tw/docs/reference).

## Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/render` | Render node tree to image |
| POST | `/render/animation` | Render animated WebP/APNG |
| POST | `/measure` | Measure layout without rendering |
| POST | `/images` | Add image to cache |
| DELETE | `/images` | Clear image cache |
| POST | `/extract-urls` | Extract resource URLs from node tree |

## Usage

### Docker

```bash
docker run -p 3000:3000 ghcr.io/flycro/takumi-http:latest
```

### Docker Compose

```yaml
services:
  takumi:
    image: ghcr.io/flycro/takumi-http:latest

  app:
    image: your-app
    environment:
      - TAKUMI_URL=http://takumi:3000
    depends_on:
      - takumi
```

With custom fonts (disabling embedded fonts):

```yaml
services:
  takumi:
    image: ghcr.io/flycro/takumi-http:latest
    environment:
      - TAKUMI_LOAD_DEFAULT_FONTS=false
    volumes:
      - ./fonts:/fonts:ro
    command: ["--font-dir", "/fonts"]
```

### Render Image

```bash
curl -X POST http://localhost:3000/render \
  -H "Content-Type: application/json" \
  -d '{
    "node": {
      "type": "container",
      "tw": "w-full h-full flex justify-center bg-black items-center",
      "style": {
        "backgroundImage": "radial-gradient(circle at 25px 25px, lightgray 2%, transparent 0%), radial-gradient(circle at 75px 75px, lightgray 2%, transparent 0%)",
        "backgroundSize": "100px 100px"
      },
      "children": [{
        "type": "container",
        "tw": "flex flex-col justify-center items-center",
        "children": [
          {
            "type": "container",
            "tw": "flex flex-row gap-3",
            "children": [
              {"type": "text", "text": "Welcome to", "tw": "text-white font-semibold text-6xl"},
              {"type": "text", "text": "Takumi", "tw": "text-[#ff3535] font-semibold text-6xl"},
              {"type": "text", "text": "Playground 👋", "tw": "text-white font-semibold text-6xl"}
            ]
          },
          {
            "type": "text",
            "text": "You can try out and experiment with Takumi here.",
            "tw": "text-white opacity-75 text-4xl mt-4",
            "style": {"fontFamily": "Geist Mono"}
          }
        ]
      }]
    },
    "options": {
      "format": "png",
      "width": 1200,
      "height": 630
    }
  }' --output playground.png
```

### Render Animation

```bash
curl -X POST http://localhost:3000/render/animation \
  -H "Content-Type: application/json" \
  -d '{
    "frames": [
      {
        "node": {
          "type": "container",
          "tw": "w-full h-full flex justify-center items-center bg-red-500",
          "children": [{"type": "text", "text": "Frame 1", "tw": "text-white text-4xl font-bold"}]
        },
        "durationMs": 500
      },
      {
        "node": {
          "type": "container",
          "tw": "w-full h-full flex justify-center items-center bg-blue-500",
          "children": [{"type": "text", "text": "Frame 2", "tw": "text-white text-4xl font-bold"}]
        },
        "durationMs": 500
      },
      {
        "node": {
          "type": "container",
          "tw": "w-full h-full flex justify-center items-center bg-green-500",
          "children": [{"type": "text", "text": "Frame 3", "tw": "text-white text-4xl font-bold"}]
        },
        "durationMs": 500
      }
    ],
    "options": {
      "format": "webp",
      "width": 400,
      "height": 200
    }
  }' --output animation.webp
```

### Multipart (file uploads)

```bash
curl -X POST http://localhost:3000/render \
  -F 'node={"type":"container","tw":"w-[400] h-[400]","children":[{"type":"image","src":"logo","tw":"w-full h-full object-cover"}]}' \
  -F 'options={"format":"png","width":400,"height":400}' \
  -F 'resource_logo=@./logo.png' \
  --output result.png
```

File uploads use the `resource_<name>` or `file_<name>` field naming convention. The `<name>` part becomes the `src` reference in your node tree.

### Measure Layout

```bash
curl -X POST http://localhost:3000/measure \
  -H "Content-Type: application/json" \
  -d '{
    "node": {
      "type": "container",
      "tw": "w-[200px] h-[100px]"
    },
    "options": {
      "width": 1000,
      "height": 1000
    }
  }'
# Returns: {"width": 200.0, "height": 100.0, ...}
```

### Image Cache

Pre-load images into the server's memory cache to reuse across multiple renders without re-uploading.

**1. Add image to cache:**

```bash
# Base64 encode your image
BASE64_IMAGE=$(base64 -w0 avatar.png)

curl -X POST http://localhost:3000/images \
  -H "Content-Type: application/json" \
  -d "{\"src\": \"user-avatar\", \"data\": \"$BASE64_IMAGE\"}"
# Returns: {"src": "user-avatar", "message": "Image added to cache"}
```

**2. Use cached image in renders:**

Reference the cached image by its `src` name in your node tree:

```bash
curl -X POST http://localhost:3000/render \
  -H "Content-Type: application/json" \
  -d '{
    "node": {
      "type": "container",
      "tw": "w-[400] h-[400] flex items-center justify-center bg-gray-100",
      "children": [{
        "type": "image",
        "src": "user-avatar",
        "tw": "w-[200] h-[200] rounded-full object-cover"
      }]
    },
    "options": {"format": "png", "width": 400, "height": 400}
  }' --output profile.png
```

The `"src": "user-avatar"` in the image node matches the `"src"` used when adding to cache.

**3. Clear cache when needed:**

```bash
curl -X DELETE http://localhost:3000/images
# Returns: {"message": "Image cache cleared", "cleared_count": 5}
```

**Image sources priority:**
1. `fetchedResources` in the request body (base64 encoded)
2. Multipart file uploads (`resource_<name>` fields)
3. Persistent image cache (`/images` endpoint)

This is useful when:
- Rendering many images with the same assets (logos, avatars, backgrounds)
- Reducing request payload size for repeated renders
- Keeping frequently used images in memory for faster access

## Configuration

| Option | Env Var | Default | Description |
|--------|---------|---------|-------------|
| `--port` | `TAKUMI_PORT` | 3000 | Server port |
| `--font-dir` | `TAKUMI_FONT_DIR` | - | Directory containing custom fonts |
| `--load-default-fonts` | `TAKUMI_LOAD_DEFAULT_FONTS` | true | Load embedded fonts (Geist, Geist Mono, Twemoji) |
| `--enable-cache` | `TAKUMI_ENABLE_CACHE` | true | Enable `/images` endpoint for pre-loading images |
| `--body-limit` | `TAKUMI_BODY_LIMIT` | 50MB | Max request body size |
| `--log-level` | `TAKUMI_LOG_LEVEL` | info | Log level (trace, debug, info, warn, error) |

## Building

```bash
cargo build --release
```

## Acknowledgments

This project is built on top of [Takumi](https://github.com/kane50613/takumi) - a high-performance image rendering engine. Thanks to [@kane50613](https://github.com/kane50613) for creating and maintaining the core library.

## License

MIT
