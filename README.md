# mkrp (Make Ren'Py Portable) 🎮

`mkrp`는 Linux ARM64 기반 레트로 핸드헬드 콘솔(RG VITA PRO, RG40XXH, RG DS 등 / ROCKNIX, Knulli, ArkOS)의 **PortMaster** 환경에서 **Ren'Py 비주얼 노벨 게임**을 쾌적하고 안정적으로 구동하기 위한 런처 템플릿 및 포팅 툴킷입니다.

---

## 🌟 주요 기능 (Features)

1. **PortMaster 표준 Ren'Py 런타임 연동**:
   - PortMaster 공식 Ren'Py 런타임 규격과 호환되는 범용 런처 스크립트(`mkrp.sh`) 및 메타데이터(`port.json`) 제공
2. **Ren'Py 8 런타임 하위 호환 레이어 (Compat Polyfills)**:
   - **Python 2 ➔ Python 3 마이그레이션 방어**: 구버전(Ren'Py 7) 게임의 스탯/호감도 대소 비교(`builtins.cmp`, `None` 비교) 충돌 방지
3. **핸드헬드 최적화 (Handheld Optimization)**:
   - 1GB~2GB 저사양 RAM 기기에서의 메모리 부족(OOM) 강제 종료를 방지하기 위한 이미지 캐시 튜닝
   - `gptokeyb`를 통한 게임패드 버튼 및 아날로그 스틱 가상 마우스 매핑 기본 제공
4. **에셋 경량화 및 리다이렉션 가이드**:
   - SDL2 미지원 비표준 이미지(AVIF 등)를 WebP/PNG로 변환 후 패치 아카이브로 온더플라이 로딩 유도

---

## 📁 프로젝트 구조 (Architecture)

### 1. 개발 저장소 구조 (Source Repository)

```text
mkrp/
├── template/              # PortMaster 배포 템플릿
│   ├── mkrp.sh            # PortMaster 기기 실행 런처 스크립트
│   ├── port.json          # PortMaster 메타데이터 및 런타임 설정
│   └── keymap.gptk        # gptokeyb 게임패드/가상 마우스 매핑 파일
├── scripts/               # 빌드 및 유틸리티 스크립트
│   ├── build.mjs          # PortMaster 배포 zip 패키징 스크립트
│   └── clean.mjs          # dist 빌드 디렉토리 정리 스크립트
├── HOW_TO_USE.md          # 기기 설치 및 실기기 세팅 가이드
├── package.json           # Node.js 프로젝트 설정
└── README.md              # 프로젝트 소개 문서
```

### 2. 배포 패키지 구조 (PortMaster `ports/` 배포 결과물)

`npm run build` 실행 시 생성되는 `dist/mkrp-v*.zip` 압축 해제 시 구조:

```text
roms/ports/ (또는 storage/roms/ports/)
├── mkrp.sh                # 게임 실행 런처
└── mkrp/                  # 게임 메인 디렉토리
    ├── port.json          # 포트 메타데이터
    ├── keymap.gptk        # 조작 매핑 파일
    ├── game/              # ⭐️ 유저 게임 에셋 (*.rpa, *.rpyc) 및 패치 스크립트
    └── saves/             # 세이브 파일 저장소
```

---

## 🚀 빠른 시작 (Quick Start)

### 1. PortMaster 배포 패키지 빌드

```bash
npm install
npm run build
```

빌드가 완료되면 `dist/mkrp-v<version>.zip` 파일이 생성됩니다.

### 2. 기기 설치 및 게임 구동

빌드된 패키지를 기기의 SD 카드 `ports/` 디렉토리에 풀고, 원하는 Ren'Py 게임의 `game/` 폴더 내용물을 복사합니다.

> [!TIP]
> 상세한 기기 설치법 및 컨트롤 조작 가이드는 [HOW_TO_USE.md](HOW_TO_USE.md) 문서를 참고해 주세요.

---

## 📄 라이선스 (License)

MIT License
