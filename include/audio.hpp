struct AudioInfo {
    uint32_t channels;
    uint32_t sample_rate;
    uint64_t frames;
};

extern "C" int get_audio_info(const char* path, AudioInfo* info);