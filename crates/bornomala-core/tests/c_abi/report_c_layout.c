#include "bornomala.h"
#include <stdio.h>
#include <stddef.h>

int main(void) {
    printf("POINTER_WIDTH:%zu\n", sizeof(void*) * 8);

    printf("SIZE:BornoEventC=%zu\n", sizeof(BornoEventC));
    printf("OFFSET:BornoEventC.event_type=%zu\n", offsetof(BornoEventC, event_type));
    printf("OFFSET:BornoEventC.char_codepoint=%zu\n", offsetof(BornoEventC, char_codepoint));
    printf("OFFSET:BornoEventC.string_payload=%zu\n", offsetof(BornoEventC, string_payload));
    printf("OFFSET:BornoEventC.index_payload=%zu\n", offsetof(BornoEventC, index_payload));
    printf("OFFSET:BornoEventC.mode_payload=%zu\n", offsetof(BornoEventC, mode_payload));

    printf("SIZE:BornoStatusC=%zu\n", sizeof(BornoStatusC));
    printf("OFFSET:BornoStatusC.name=%zu\n", offsetof(BornoStatusC, name));
    printf("OFFSET:BornoStatusC.version=%zu\n", offsetof(BornoStatusC, version));
    printf("OFFSET:BornoStatusC.active_mode=%zu\n", offsetof(BornoStatusC, active_mode));
    printf("OFFSET:BornoStatusC.ready=%zu\n", offsetof(BornoStatusC, ready));

    printf("SIZE:BornoStateC=%zu\n", sizeof(BornoStateC));
    printf("OFFSET:BornoStateC.preedit=%zu\n", offsetof(BornoStateC, preedit));
    printf("OFFSET:BornoStateC.raw_buffer=%zu\n", offsetof(BornoStateC, raw_buffer));
    printf("OFFSET:BornoStateC.cursor_pos=%zu\n", offsetof(BornoStateC, cursor_pos));
    printf("OFFSET:BornoStateC.candidates_count=%zu\n", offsetof(BornoStateC, candidates_count));
    printf("OFFSET:BornoStateC.candidates=%zu\n", offsetof(BornoStateC, candidates));
    printf("OFFSET:BornoStateC.selected_candidate_index=%zu\n", offsetof(BornoStateC, selected_candidate_index));
    printf("OFFSET:BornoStateC.is_composing=%zu\n", offsetof(BornoStateC, is_composing));
    printf("OFFSET:BornoStateC.mode=%zu\n", offsetof(BornoStateC, mode));

    printf("SIZE:BornoProcessResultC=%zu\n", sizeof(BornoProcessResultC));
    printf("OFFSET:BornoProcessResultC.state=%zu\n", offsetof(BornoProcessResultC, state));
    printf("OFFSET:BornoProcessResultC.committed_text=%zu\n", offsetof(BornoProcessResultC, committed_text));
    printf("OFFSET:BornoProcessResultC.cancelled=%zu\n", offsetof(BornoProcessResultC, cancelled));
    printf("OFFSET:BornoProcessResultC.handled=%zu\n", offsetof(BornoProcessResultC, handled));
    return 0;
}

