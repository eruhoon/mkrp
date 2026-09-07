# mkrp (Make Ren'Py Portable) 🎮

`mkrp`는 Linux ARM64 기반 레트로 핸드헬드 콘솔(RG VITA PRO, RG40XXH, RG DS 등 / ROCKNIX, Knulli, PortMaster)에서 **Ren'Py 비주얼 노벨 게임**을 쾌적하고 안정적으로 구동하기 위한 툴킷 및 유틸리티 프로젝트입니다.

---

## 🌟 주요 기능 (Features)

1. **RPA 아카이브 처리 및 복구 유틸리티 (`mkrp-rpa`)**:
   - Ren'Py 아카이브 포맷(RPA-2.0, RPA-3.0, RPA-3.2) 완벽 파싱 및 인덱스 추출
   - 대용량 에셋 패치 파일 생성 및 헤더 키(Key) 슬라이스 정렬 보정
2. **Ren'Py 8 런타임 호환성 보강 (Compat Polyfills)**:
   - **Python 2 ➔ Python 3 마이그레이션 호환 레이어**:
     - `builtins.cmp` 복원 및 Python 2 스타일 대소 비교(`None` 비교 규칙, `.value` 자동 언래핑)
     - 구버전 렌파이(Ren'Py 7) 스탯 시스템 및 시간 시스템 충돌 방어
3. **에셋 변환 및 가상 로더 (Transparent Asset Loader)**:
   - 비표준/고용량 포맷(AVIF 등)을 경량 PNG/WebP로 온더플라이 리다이렉션하여 렌더링 호환성 확보
4. **핸드헬드 최적화 (Handheld Optimization)**:
   - 1GB~2GB 저사양 RAM 기기를 위한 이미지 캐시 크기 최적화 (`config.image_cache_size_mb`)
   - 게임패드 기본 활성화 및 가상 마우스 매핑 지원

---

## 📁 프로젝트 구조 (Architecture)

```text
mkrp/
├── crates/
│   ├── mkrp-rpa/          # Ren'Py RPA 아카이브 파서 및 빌더 코어
│   ├── mkrp-render/       # 렌더링 및 에셋 최적화 라이브러리
│   └── mkrp-app/          # CLI 유틸리티 (inspect_patch_rpa, fix_patch_rpa 등)
├── HOW_TO_USE.md          # 핸드헬드 기기 실기기 사용 및 설치 가이드
└── README.md              # 프로젝트 소개 문서
```

---

## 🚀 빠른 시작 (Quick Start)

자세한 기기 설정 및 게임 설치 방법은 [HOW_TO_USE.md](HOW_TO_USE.md) 문서를 참고해 주세요.

### RPA 검사 및 패치 툴 빌드

```bash
cargo build --release
```

- 아카이브 인덱스 검사: `cargo run --bin inspect_patch_rpa`
- 패치 아카이브 빌드: `cargo run --bin build_patch_rpa`

---

## 📄 라이선스 (License)

MIT License
