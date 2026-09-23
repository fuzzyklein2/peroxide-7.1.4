#define MINIAUDIO_IMPLEMENTATION
#include "miniaudio.h"

#include "audio.hpp"

int get_audio_info(const char* path, AudioInfo* info)
{
    ma_decoder decoder;
    if (ma_decoder_init_file(path, nullptr, &decoder) != MA_SUCCESS)
        return 1;

    info->channels = decoder.outputChannels;
    info->sample_rate = decoder.outputSampleRate;

    ma_uint64 totalFrames;

    ma_decoder_get_length_in_pcm_frames(&decoder, &totalFrames);

    info->frames = totalFrames;

    return 0;
}